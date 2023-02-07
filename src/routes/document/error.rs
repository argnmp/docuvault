use axum::{response::IntoResponse, http::StatusCode};

#[derive(Debug)]
pub enum DocumentError {
    ScopeNotExist,
}
impl IntoResponse for DocumentError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::ScopeNotExist => (StatusCode::BAD_REQUEST, "specified scope does not exists."),
        };
        res.into_response()
    }
}
