use anyhow::{anyhow, Result};
use lettre::{
    message::{Mailbox, MultiPart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::graphql::models::user::User;

pub async fn send_email_verification_code(user: &User) -> Result<()> {
    let from: Mailbox = format!("{} <{}>", "connefut", "info@connefut.com").parse()?;
    let to: Mailbox = user.email.as_str().parse()?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("メールアドレスの認証コードを送信しました")
        .multipart(MultiPart::alternative_plain_html(
            String::from("hello!"),
            include_str!("./template/email_verification_code.html").replace(
                "{code}",
                match user.email_verification_code {
                    Some(ref code) => code.as_str(),
                    None => return Err(anyhow!("Email address verification code is not set.")),
                },
            ),
        ))?;

    let creds = Credentials::new("user".to_string(), "user".to_string());
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("mailhog")
            .port(1025)
            .credentials(creds)
            .build();

    match mailer.send(email).await {
        Ok(_) => {
            tracing::info!("email send successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e.into())
        }
    }
}
