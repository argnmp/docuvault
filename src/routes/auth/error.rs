use axum::{
    response::IntoResponse,
    http::StatusCode,
};

#[derive(Debug)]
pub enum AuthError {
    MissingCredential,
}
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::MissingCredential => (StatusCode::BAD_REQUEST, "missing credential"),
        };
        res.into_response()
    }
}
