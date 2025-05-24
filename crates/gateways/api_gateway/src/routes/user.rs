use axum::Router;

pub fn user_routes() -> Router {
    Router::new().nest(CREA, router)
}
