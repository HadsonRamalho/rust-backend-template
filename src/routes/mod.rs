use crate::controllers::jwt::jwt_auth;
use crate::controllers::utils::get_database_url_from_env;
use crate::models::error::ApiError;
use crate::routes::docs::get_api_docs;
use crate::routes::user::user_routes;
use axum::{Json, Router};
use axum::{
    extract::DefaultBodyLimit,
    http::HeaderValue,
    middleware,
    routing::{get, get_service},
};
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use hyper::StatusCode;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

pub mod docs;
pub mod user;

pub async fn print_protected_route()
-> Result<(StatusCode, Json<String>), (StatusCode, Json<ApiError>)> {
    Ok((StatusCode::OK, Json("Protected route!".to_string())))
}

#[axum::debug_handler]
pub async fn print_common_route() -> Result<(StatusCode, Json<String>), (StatusCode, Json<ApiError>)>
{
    Ok((StatusCode::OK, Json("Common route!".to_string())))
}

pub fn protected_routes(pool: Pool<AsyncPgConnection>) -> OpenApiRouter<Pool<AsyncPgConnection>> {
    let protected_routes = OpenApiRouter::new()
        .route("/protected", get(print_protected_route))
        .layer(middleware::from_fn_with_state(pool.clone(), jwt_auth))
        .with_state(pool);

    protected_routes
}

pub fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_config = ClientConfig::with_platform_verifier();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config.unwrap());
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };
    fut.boxed()
}

pub async fn init_routes() -> Router {
    let db_url = get_database_url_from_env().ok();

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    if let Some(db_url) = db_url {
        let mgr =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(db_url, config);
        let pool = Pool::builder(mgr).max_size(10).build().unwrap();

        let app: OpenApiRouter<_> = OpenApiRouter::new()
            .route("/common", get(print_common_route))
            .nest_service("/images", get_service(ServeDir::new("./images")));

        return Router::new()
            .nest("/api", app.into())
            .nest("/api/user", user_routes().await.into())
            .nest("/api", protected_routes(pool.clone()).into())
            .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", get_api_docs()))
            .with_state(pool)
            .layer(DefaultBodyLimit::max(1024 * 1024 * 100))
            .layer(
                CorsLayer::new()
                    .allow_origin(vec![
                        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
                    ])
                    .allow_methods(Any)
                    .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]),
            );
    }
    Router::new()
}
