use once_cell::sync::Lazy;
use redis::AsyncCommands;
use sea_orm::{entity::*, query::*};

use crate::{AppState, entity, modules::redis::redis_reset_scopes};


pub async fn bootstrap(state: AppState) {
    redis_reset_scopes(state.clone()).await;
}

