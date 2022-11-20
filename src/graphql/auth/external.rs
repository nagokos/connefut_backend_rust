use serde::{Deserialize, Serialize};

pub mod google;
pub mod line;

#[derive(sqlx::Type, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "authentication_provider")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthenticationProvider {
    Google,
    Line,
}

// todo providerフィールド作ってないがいい方法探す 関数にenumで渡しているため
#[derive(Deserialize, Debug, Serialize)]
pub struct UserInfo {
    pub sub: String,
    pub name: String,
    pub email: String,
    pub picture: Option<String>,
}
