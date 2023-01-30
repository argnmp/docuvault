use axum::{
    response::IntoResponse,
    http::StatusCode,
};

#[derive(Debug)]
pub enum GlobalError {
    InternalServerError,
    NoPermission,
}
impl IntoResponse for GlobalError {
    fn into_response(self) -> axum::response::Response {
        let res = match self {
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
            Self::NoPermission => (StatusCode::FORBIDDEN, "no permission"),
        };
        res.into_response()
    }
}
