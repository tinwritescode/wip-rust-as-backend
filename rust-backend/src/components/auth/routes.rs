use super::{
    create_token, create_user, fetch_user_by_email, fetch_user_by_email_and_password, LoginUser,
};
use crate::{
    base::BaseResponse,
    components::auth::{model::NewUser, AccessToken, RefreshToken, User, UserWithTokens},
    error_handler::ResponseError,
    DataResult, ResponseSuccess,
};
use okapi::openapi3::OpenApi;
use rocket::{http::Status, serde::json::Json, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec};
use schemars::_serde_json;
use validator::Validate;

pub fn routes() -> (std::vec::Vec<Route>, OpenApi) {
    openapi_get_routes_spec![register, login, refresh]
}

#[openapi(tag = "Auth")]
#[post("/register", data = "<user>")]
fn register(user: DataResult<'_, NewUser>) -> BaseResponse<User> {
    let user = user.expect("Failed to parse user");
    if let Err(e) = user.validate() {
        return Err((
            Status::BadRequest,
            Json(ResponseError::new_with_data(
                e.to_string(),
                Some(
                    e.into_errors()
                        .into_iter()
                        .map(|e| _serde_json::json!(e))
                        .collect(),
                ),
            )),
        ));
    }
    let conn = &mut crate::establish_connection();

    let insert = match create_user(&user, conn) {
        Ok(insert) => insert,
        Err(err) => {
            return Err(err);
        }
    };

    if insert != 1 {
        return Err((
            Status::InternalServerError,
            Json(ResponseError::new("Failed to create user".to_string())),
        ));
    }

    Ok(Json(ResponseSuccess::new(
        fetch_user_by_email(user.email, conn).unwrap().user,
    )))
}

#[openapi(tag = "Auth")]
#[post("/login", data = "<user>")]
fn login(user: DataResult<'_, LoginUser>) -> BaseResponse<UserWithTokens> {
    let user = user.expect("Failed to parse user");
    if let Err(e) = user.validate() {
        println!("{:?}", e.errors());
        println!("{}", _serde_json::json!(e));

        return Err((
            Status::BadRequest,
            Json(ResponseError::new_with_data(
                e.to_string(),
                Some(_serde_json::json!(e)),
            )),
        ));
    }
    let conn = &mut crate::establish_connection();

    let user = match fetch_user_by_email_and_password(user.email, user.password, conn) {
        Ok(user) => user,
        Err(err) => {
            return Err(err);
        }
    };

    let (access_token, refresh_token) = (
        create_token(user.user.id, super::TokenType::AccessToken, conn),
        create_token(user.user.id, super::TokenType::RefreshToken, conn),
    );

    match (access_token, refresh_token) {
        (Ok(access_token), Ok(refresh_token)) => Ok(Json(ResponseSuccess::new(UserWithTokens {
            user: user.user,
            access_token,
            refresh_token,
        }))),
        (Err(err), _) => Err(err),
        (_, Err(err)) => Err(err),
    }
}

#[openapi(tag = "Auth")]
#[post("/refresh", data = "<refresh_token>")]
fn refresh(refresh_token: DataResult<'_, RefreshToken>) -> BaseResponse<AccessToken> {
    let refresh_token = refresh_token.expect("Failed to parse refresh token");
    let conn = &mut crate::establish_connection();
    let token = crate::components::auth::service::fetch_token(&refresh_token.refresh_token, conn);

    match token {
        Ok(token) => {
            let access_token = create_token(token.user_id, super::TokenType::AccessToken, conn);

            match access_token {
                Ok(access_token) => Ok(Json(ResponseSuccess::new(AccessToken { access_token }))),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
