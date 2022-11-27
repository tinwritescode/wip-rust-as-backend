use crate::schema::*;
use diesel::{data_types::PgTimestamp, Identifiable, Insertable, Queryable, Selectable};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Debug, PartialEq, Eq, Identifiable, Selectable, JsonSchema)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Queryable)]
pub struct UserWithPassword {
    pub user: User,
    pub password: String,
}

#[derive(FromForm, Insertable, Clone, Copy, Debug, JsonSchema)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(FromForm, JsonSchema)]
pub struct LoginUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

pub enum TokenType {
    AccessToken,
    RefreshToken,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Insertable)]
#[diesel(table_name = tokens)]
pub struct NewRefreshToken {
    pub user_id: i32,
    pub token: String,
}

#[derive(Serialize, JsonSchema)]
pub struct UserWithTokens {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Queryable, Selectable)]
pub struct Token {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expired_at: PgTimestamp,
}

#[derive(FromForm, JsonSchema)]
pub struct RefreshToken {
    pub refresh_token: String,
}

#[derive(Serialize, JsonSchema)]
pub struct AccessToken {
    pub access_token: String,
}
