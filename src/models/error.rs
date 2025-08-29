use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum ApiError {
    #[error("Error processing your request: {0}")]
    Request(String),

    #[error("Error while trying to connect to the database: {0}")]
    DatabaseConnection(String),

    #[error("Invalid authorization token")]
    InvalidAuthorizationToken,

    #[error("Multiple errors while validating the authorization token: {0:?}")]
    MultipleAuthorizationErrors(Vec<String>),

    #[error("A database error occurred: {0}")]
    Database(String),

    #[error("Failed to create token: {0}")]
    CreateToken(String),

    #[error("Missing fields in the request")]
    InvalidData,

    #[error("Invalid email provided")]
    InvalidEmail,

    #[error("User not found by email")]
    EmailNotFound,

    #[error("User is not active")]
    NotActiveUser,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Missing frontend URL")]
    FrontendUrl,

    #[error("User not found")]
    UserNotFound,
}
