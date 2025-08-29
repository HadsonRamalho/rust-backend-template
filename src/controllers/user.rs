use axum::{Json, extract::State};
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use hyper::{HeaderMap, StatusCode};
use pwhash::bcrypt::verify;

use crate::{
    controllers::{
        jwt::{extract_claims_from_header, generate_jwt},
        utils::get_conn,
    },
    models::{
        self,
        error::ApiError,
        user::{LoginUser, RegisterUser, UpdateUser, User, UserAuthInfo},
    },
};
#[utoipa::path(post, path = "/user/register", responses((status = CREATED, body = RegisterUser)))]
pub async fn api_register_user(
    State(pool): State<Pool<AsyncPgConnection>>,
    input: Json<RegisterUser>,
) -> Result<StatusCode, (StatusCode, Json<String>)> {
    let mut user_input = input.0;
    if !user_input.validate_fields() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError::InvalidData.to_string()),
        ));
    }

    match user_input.parse_fields() {
        Ok(_) => {}
        Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e))),
    }

    let user = User::from(user_input);

    let conn = &mut get_conn(&pool).await?;

    match models::user::register_user(conn, &user).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e))),
    }
}

#[utoipa::path(post, path = "/user/login", responses((status = OK, body = LoginUser)))]
pub async fn api_login_user(
    State(pool): State<Pool<AsyncPgConnection>>,
    input: Json<LoginUser>,
) -> Result<(StatusCode, Json<String>), (StatusCode, Json<String>)> {
    let mut user_input = input.0;

    match user_input.validate_fields() {
        Ok(_) => {}
        Err(e) => return Err((StatusCode::BAD_REQUEST, Json(e))),
    }
    user_input.parse_fields();

    let conn = &mut get_conn(&pool).await?;

    let user = match models::user::find_user_by_email(conn, &user_input.email).await {
        Ok(user) => user,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::EmailNotFound.to_string()),
            ));
        }
    };

    if !user.is_active || user.deletion_date.is_some() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::NotActiveUser.to_string()),
        ));
    }

    let token = generate_jwt(UserAuthInfo::from(user.clone()))?;

    if verify(user_input.password, &user.password) {
        return Ok((StatusCode::OK, Json(token)));
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError::InvalidPassword.to_string()),
    ))
}

pub async fn api_update_user_data(
    State(pool): State<Pool<AsyncPgConnection>>,
    headers: HeaderMap,
    input: Json<UpdateUser>,
) -> Result<StatusCode, (StatusCode, Json<String>)> {
    let id = extract_claims_from_header(&headers).await?.1.id;

    let conn = &mut get_conn(&pool).await?;

    match models::user::find_user_by_id(conn, &id).await {
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::UserNotFound.to_string()),
            ));
        }
        _ => {}
    };

    let update_data = input.0;

    match models::user::update_user_data(conn, &id, &update_data).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string()))),
    }
}
