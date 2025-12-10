use crate::AppState;

pub mod v2;

pub fn router() -> axum::Router<AppState> {
    let v2 = v2::router();
    axum::Router::new().nest("/v2", v2)
}
