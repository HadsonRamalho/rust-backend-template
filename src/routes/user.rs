use axum::routing::{patch, post};
use diesel_async::{AsyncPgConnection, pooled_connection::deadpool::Pool};
use utoipa_axum::router::OpenApiRouter;

use crate::controllers::user::{api_login_user, api_register_user, api_update_user_data};

pub async fn user_routes() -> OpenApiRouter<Pool<AsyncPgConnection>> {
    let routes = OpenApiRouter::new()
        .route("/register", post(api_register_user))
        .route("/login", post(api_login_user))
        .route("/update", patch(api_update_user_data));

    routes
}
