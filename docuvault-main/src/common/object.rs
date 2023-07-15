use std::sync::Arc;
use axum::extract::FromRef;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::DatabaseConnection;

use crate::{AppState, routes::auth::service::AuthService};

#[derive(Clone, Debug)]
pub struct ServiceState<T>{
    pub global_state: AppState,
    pub service: Arc<T>,
}
impl<T> FromRef<ServiceState<T>> for DatabaseConnection {
    fn from_ref(input: &ServiceState<T>) -> Self {
        input.global_state.db_conn.clone()
    } 
}
impl<T> FromRef<ServiceState<T>> for Pool<RedisConnectionManager> {
    fn from_ref(input: &ServiceState<T>) -> Self {
        input.global_state.redis_conn.clone()
    }
}
