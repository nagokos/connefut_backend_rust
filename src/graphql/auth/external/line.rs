use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use async_graphql::futures_util::TryFutureExt;
use axum::{response::Redirect, Extension};
use axum_extra::extract::{cookie::Cookie, CookieJar, Query};
use cookie::{time::Duration, SameSite};
use hyper::StatusCode;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use sqlx::PgPool;

use crate::{
    config::{Config, Line},
    graphql::{
        auth::{
            external::UserInfo,
            jwt::{self, Claims},
        },
        models::{
            authentication::{create_authentication, get_external_user_from_provider_and_uid},
            user::{create_with_external_certification, is_already_exists_email},
        },
    },
};

use super::AuthenticationProvider;

pub struct LineAuth {
    client: CoreClient,
}

pub async fn new_line_auth_client(line: &Line) -> Result<LineAuth> {
    let line_client_id = ClientId::new(line.client_id.to_owned());
    let line_client_secret = ClientSecret::new(line.client_secret.to_owned());

    let line_redirect_url = RedirectUrl::new(line.callback_url.to_owned()).map_err(|e| {
        tracing::error!("RedirectUrl new failed: {:?}", e);
        anyhow!("RedirectUrl new faile")
    })?;

    let issuer_url = IssuerUrl::new("https://access.line.me".to_string()).map_err(|e| {
        tracing::error!("Invalid issuer URL: {:?}", e);
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
        line_client_id,
        Some(line_client_secret),
    )
    .set_redirect_uri(line_redirect_url);
    Ok(LineAuth { client })
}

pub async fn auth_line_redirect(
    jar: CookieJar,
    Extension(auth): Extension<Arc<LineAuth>>,
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

pub async fn auth_line_callback(
    jar: CookieJar,
    Extension(auth): Extension<Arc<LineAuth>>,
    Extension(config): Extension<&Config>,
    Extension(pool): Extension<Arc<PgPool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<(CookieJar, Redirect), StatusCode> {
    // cookieのstateとparamsのstateを取得して比較(csrf対策)
    let cookie_state = jar.get("state").ok_or_else(|| {
        tracing::error!("state not found in the cookie");
        StatusCode::BAD_REQUEST
    })?;
    let state = params.get("state").ok_or_else(|| {
        tracing::error!("state not found in the params");
        StatusCode::BAD_REQUEST
    })?;
    if cookie_state.value() != state {
        tracing::error!("state does not match");
        return Err(StatusCode::BAD_REQUEST);
    }

    // 認可コードを取得
    let code = params.get("code").ok_or_else(|| {
        tracing::error!("authorization code not found in the params");
        StatusCode::BAD_REQUEST
    })?;
    // pkce_verifierを取得
    let pkce_verifier = jar.get("pkce_verifier").ok_or_else(|| {
        tracing::error!("pkce_verifier not found in the cookie");
        StatusCode::BAD_REQUEST
    })?;

    // pkce_challengeが認可サーバー側で保存してあるのでcodeとpkce_verrifierを加えてトークンリクエストを投げる
    // 整合性がチェックされたらトークンが返される
    let token_response = auth
        .client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.value().to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            tracing::error!("token request failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let id_token = token_response.id_token().ok_or_else(|| {
        tracing::error!("Server did not return an ID token");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let nonce = jar.get("nonce").ok_or_else(|| {
        tracing::error!("nonce not found in the cookie");
        StatusCode::BAD_REQUEST
    })?;

    // LINEログインAPIのエンドポイントを利用
    // トークン検証のためにはid_token client_id nonceが必要
    let params = [
        ("id_token", id_token.to_string()),
        ("client_id", config.line.client_id.clone()),
        ("nonce", nonce.value().to_owned()),
    ];
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.line.me/oauth2/v2.1/verify")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(
                "Token validation using the LINE Login API endpoint failed: {:?}",
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user_info = response
        .json::<UserInfo>()
        .map_err(|e| {
            tracing::error!("user info deserialization failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .await?;

    // 外部認証を利用して登録したユーザーが存在したらそのユーザーでログイン
    // 外部認証を利用して未登録の場合は取得したclaimsのユーザー情報でユーザーを作成
    // 外部認証を利用したユーザーはprovider(google or line)とuid(sub)の組み合わせで識別する
    // 外部認証を利用して登録したユーザーを取得
    match get_external_user_from_provider_and_uid(
        &pool,
        &user_info.sub,
        AuthenticationProvider::Line,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    .await?
    {
        // 既に外部認証で登録済みの場合
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
        // 外部認証で未登録の場合
        None => {
            // メールアドレスの存在チェック
            if is_already_exists_email(&user_info.email, &pool)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                .await?
            {
                tracing::error!("this email already exists");
                return Err(StatusCode::BAD_REQUEST);
            }

            // トランザクションの開始
            let mut tx = pool
                .begin()
                .map_err(|e| {
                    tracing::error!("transaction begin failed: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .await?;

            // 外部認証を利用してユーザーを作成
            let user = match create_with_external_certification(&mut tx, &user_info).await {
                Ok(user) => user,
                Err(_) => {
                    // エラーの場合はrollback
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

            // 認証情報の作成
            if create_authentication(
                &mut tx,
                &user_info.sub,
                AuthenticationProvider::Line,
                user.id,
            )
            .await
            .is_err()
            {
                // エラーの場合はrollback
                tracing::error!("create authentication failed transaction rollback");
                tx.rollback()
                    .map_err(|e| {
                        tracing::error!("transaction rollback failed: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
                    .await?;
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            // コミットする
            tx.commit()
                .map_err(|e| {
                    tracing::error!("transaction commit failed: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .await?;

            // jwt tokenの作成
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

    // アクセストークンの検証
    // userinfoに追加でリクエスト投げる場合は必要
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
