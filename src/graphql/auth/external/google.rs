use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::extract::Query;
use axum::{response::Redirect, Extension};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use cookie::time::Duration;
use cookie::SameSite;
use futures::TryFutureExt;
use hyper::StatusCode;
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreProviderMetadata,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use sqlx::PgPool;

use crate::config::Google;
use crate::graphql::auth::external::UserInfo;
use crate::graphql::auth::jwt::{self, Claims};
use crate::graphql::models::authentication::{
    create_authentication, get_external_user_from_provider_and_uid,
};
use crate::graphql::models::user::{create_with_external_certification, is_already_exists_email};

use super::AuthenticationProvider;

#[derive(Debug)]
pub struct GoogleAuth {
    client: CoreClient,
}

pub async fn new_google_auth_client(google: &Google) -> Result<GoogleAuth> {
    let google_client_id = ClientId::new(google.client_id.clone());
    let google_client_secret = ClientSecret::new(google.client_secret.clone());

    let google_redirect_url = RedirectUrl::new(google.callback_url.clone()).map_err(|e| {
        tracing::error!("redirect url new error: {:?}", e);
        anyhow!("Redirect url new error")
    })?;

    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string()).map_err(|e| {
        tracing::error!("Invalid issuer URL: {}", e);
        anyhow!("Invalid issuer URL")
    })?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client)
        .map_err(|e| {
            tracing::error!("Failed to discover OpenID Provider: {:?}", e);
            anyhow!("Failed to discover OpenID Provider")
        })
        .await?;

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(google_redirect_url);
    Ok(GoogleAuth { client })
}

pub async fn auth_google_redirect(
    Extension(auth): Extension<Arc<GoogleAuth>>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), StatusCode> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_state, nonce) = auth
        .client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // todo ?????????????????????????????????
    let pkce_cookie = Cookie::build("pkce_verifier", pkce_verifier.secret().to_string())
        .max_age(Duration::hours(1))
        .http_only(true)
        .finish();
    let csrf_cookie = Cookie::build("state", csrf_state.secret().to_string())
        .max_age(Duration::hours(1))
        .http_only(true)
        .finish();
    let nonce_cookie = Cookie::build("nonce", nonce.secret().to_string())
        .max_age(Duration::hours(1))
        .http_only(true)
        .finish();

    Ok((
        jar.add(pkce_cookie).add(csrf_cookie).add(nonce_cookie),
        Redirect::to(auth_url.as_str()),
    ))
}

pub async fn auth_google_callback(
    jar: CookieJar,
    Extension(auth): Extension<Arc<GoogleAuth>>,
    Extension(pool): Extension<Arc<PgPool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    // state????????? csrf???????????????????????????
    let state_cookie = jar.get("state").ok_or_else(|| {
        tracing::error!("state not found in the cookie");
        StatusCode::BAD_REQUEST
    })?;
    let state = params.get("state").ok_or_else(|| {
        tracing::error!("state not found in the params");
        StatusCode::BAD_REQUEST
    })?;
    if state_cookie.value() != state {
        tracing::error!("state does not match");
        return Err(StatusCode::BAD_REQUEST);
    }

    // code(???????????????)?????????
    let code = params.get("code").ok_or_else(|| {
        tracing::error!("authorization code not found in the params");
        StatusCode::BAD_REQUEST
    })?;
    // pkce_verifier?????????
    let pkce_verifier = jar.get("pkce_verifier").ok_or_else(|| {
        tracing::error!("pkce_verifier not found in the cookie");
        StatusCode::BAD_REQUEST
    })?;
    // pkce_challenge??????????????????????????????????????????????????? code???pkce_verrifier???????????????????????????????????????????????????
    // ???????????????????????????????????????????????????????????????
    let token_response = auth
        .client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.value().to_string()))
        .request_async(async_http_client)
        .map_err(|e| {
            tracing::error!("token request failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .await?;

    // id_token?????????
    let id_token = token_response.id_token().ok_or_else(|| {
        tracing::error!("Server did not return an ID token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // nonce?????????
    let nonce = jar.get("nonce").ok_or_else(|| {
        tracing::error!("No nonce in the cookie");
        StatusCode::BAD_REQUEST
    })?;
    // ID???????????????????????????nonce?????????
    let claims: &CoreIdTokenClaims = id_token
        .claims(
            &auth.client.id_token_verifier(),
            &Nonce::new(nonce.value().to_string()),
        )
        .map_err(|e| {
            tracing::error!("id_token verify failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // fn user_info_new(claims)??????????????????????????????????????????????????????
    let sub = claims.subject().to_string();

    let name = claims
        .name()
        // and_then????????????flat?????????Option<T> map??????Option<Option<T>>
        .and_then(|name| name.get(None).map(|name| name.to_string()))
        .ok_or_else(|| {
            tracing::error!("Name is required in Claims");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let email = claims
        .email()
        .map(|email| email.to_string())
        .ok_or_else(|| {
            tracing::error!("Email is required in Claims");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // picture??????????????????option<string>???????????????
    let picture = claims
        .picture()
        .and_then(|picture| picture.get(None).map(|picture| picture.to_string()));

    let user_info = UserInfo {
        sub,
        name,
        email,
        picture,
    };

    // ?????????????????????????????????????????????
    // ????????????????????????Some(User)
    match get_external_user_from_provider_and_uid(
        &pool,
        &user_info.sub,
        AuthenticationProvider::Google,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .await?
    {
        // ?????????????????????????????????????????????????????????????????????
        Some(user) => {
            let claims = Claims {
                sub: user.id.to_string(),
                ..Default::default()
            };
            let jwt = jwt::token_encode(claims).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let cookie = Cookie::build("token", jwt)
                .http_only(true)
                .max_age(Duration::DAY)
                .same_site(SameSite::Lax)
                .finish();

            Ok((jar.add(cookie), Redirect::to("http://google.com")))
        }
        // ???????????????????????????
        None => {
            // ????????????????????????????????????????????????????????????????????????
            // ??????????????????????????????????????????
            if is_already_exists_email(&user_info.email, &pool)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                .await?
            {
                tracing::error!("this email already exists");
                return Err(StatusCode::BAD_REQUEST);
            }

            // ??????????????????????????????
            let mut tx = pool
                .begin()
                .map_err(|e| {
                    tracing::error!("{:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .await?;

            // claims????????????????????????????????????????????????
            let user = match create_with_external_certification(&mut tx, &user_info).await {
                Ok(user) => user,
                Err(_) => {
                    // ???????????????rollback
                    tracing::error!(
                        "create with external cetification failed transaction rollback"
                    );
                    tx.rollback()
                        .map_err(|e| {
                            tracing::error!("transaction rollback failed: {:?}", e);
                            StatusCode::INTERNAL_SERVER_ERROR
                        })
                        .await?;
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // ????????????????????????????????????????????????(provider???sub???????????????)
            if create_authentication(
                &mut tx,
                &user_info.sub,
                AuthenticationProvider::Google,
                user.id,
            )
            .await
            .is_err()
            {
                // ???????????????rollback
                tracing::error!("create authentication failed transaction rollback");
                tx.rollback()
                    .map_err(|e| {
                        tracing::error!("transaction rollback failed: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                    .await?;
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            // commit??????
            tx.commit()
                .map_err(|e| {
                    tracing::error!("transaction commit failed: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .await?;

            // ???????????????????????????????????????
            let claims = Claims {
                sub: user.id.to_string(),
                ..Default::default()
            };
            let jwt = jwt::token_encode(claims).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let cookie = Cookie::build("token", jwt)
                .http_only(true)
                .max_age(Duration::DAY)
                .same_site(SameSite::Lax)
                .finish();
            Ok((jar.add(cookie), Redirect::to("http://google.com")))
        }
    }

    // ?????????????????????????????????
    // userinfo???????????????????????????????????????????????????
    //if let Some(expected_access_token_hash) = claims.access_token_hash() {
    //    let actual_access_token_hash = AccessTokenHash::from_token(
    //        token_response.access_token(),
    //        &id_token.signing_alg().map_err(|e| {
    //            tracing::error!("id_token signing_alg failed: {:?}", e);
    //            StatusCode::INTERNAL_SERVER_ERROR
    //        })?,
    //    )
    //    .map_err(|e| {
    //        tracing::error!("Invalid access token: {:?}", e);
    //        StatusCode::INTERNAL_SERVER_ERROR
    //    })?;
    //    if actual_access_token_hash != *expected_access_token_hash {
    //        tracing::error!("Invalid access token");
    //        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    //    }
    //}
}
