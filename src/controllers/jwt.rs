use std::env;

use crate::models::{error::ApiError, jwt::Claims, user::UserAuthInfo};
use axum::{Json, body::Body, extract::Request, middleware::Next, response::Response};
use dotenvy::dotenv;
use hyper::{HeaderMap, StatusCode};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode};

pub async fn jwt_auth(
    req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<String>)> {
    let _ = extract_claims_from_header(req.headers()).await?;
    Ok(next.run(req).await)
}

pub async fn extract_claims_from_header(
    headers: &HeaderMap,
) -> Result<(String, Claims), (StatusCode, Json<String>)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            Some(header.trim_start_matches("Bearer ").trim())
        }
        _ => None,
    };

    let token = match token {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError::InvalidAuthorizationToken.to_string()),
            ));
        }
    };

    let secret = get_jwt_secret_from_env()?;

    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );

    let claims = match decoded {
        Ok(data) => (token.to_string(), data.claims),
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError::InvalidAuthorizationToken.to_string()),
            ));
        }
    };

    let _ = validate_claims(&claims.1).await?;

    Ok(claims)
}

pub async fn validate_claims(claims: &Claims) -> Result<StatusCode, (StatusCode, Json<String>)> {
    let mut errors = vec![];

    if claims.id.to_string().trim().is_empty() {
        errors.push("Invalid ID".to_string())
    }
    if claims.public_id.to_string().is_empty() {
        errors.push("Invalid Public ID".to_string())
    }
    if claims.email.is_empty() {
        errors.push("Invalid E-mail".to_string())
    }
    if claims.exp == 0 {
        errors.push("Invalid expiration date".to_string())
    }

    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp < now {
        errors.push("Expired token".to_string())
    }

    if errors.is_empty() {
        return Ok(StatusCode::OK);
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(ApiError::MultipleAuthorizationErrors(errors).to_string()),
    ))
}

pub fn get_jwt_secret_from_env() -> Result<String, (StatusCode, Json<String>)> {
    dotenv().ok();

    match env::var("JWT_SECRET") {
        Ok(secret) => Ok(secret),
        Err(error) => {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiError::DatabaseConnection(error.to_string()).to_string()),
            ));
        }
    }
}

pub fn generate_jwt(input: UserAuthInfo) -> Result<String, (StatusCode, Json<String>)> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .expect("Invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        id: input.id,
        email: input.email.to_string(),
        exp: expiration,
        public_id: input.public_id,
        user_type: input.user_type,
    };

    let secret = get_jwt_secret_from_env()?;

    match jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => Ok(token),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError::CreateToken(e.to_string()).to_string()),
        )),
    }
}
