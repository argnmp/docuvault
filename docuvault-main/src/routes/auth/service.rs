use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sea_orm::{entity:: *, query::*, DbErr};
use crate::{entity, AppState, routes::error::GlobalError, db::schema::redis::{RedisSchemaHeader, BlackList, TokenPair, Refresh}};

#[derive(Clone, Debug)]
pub struct AuthService{
    state: AppState
}

impl AuthService {
    pub fn new(shared_state: AppState) -> Self{
        Self {
            state: shared_state,
        }
    }  
    pub async fn find_user(&self, email: &str)->Result<Option<entity::docuser::Model>, GlobalError>{
        let qr = entity::docuser::Entity::find()
            .filter(entity::docuser::Column::Email.eq(email))
            .one(&self.state.db_conn)
            .await?;

        Ok(qr)
        
    }
    pub async fn find_users(&self, email: &str)->Result<Option<Vec<entity::docuser::Model>>, GlobalError>{
        let qr = entity::docuser::Entity::find()
            .filter(entity::docuser::Column::Email.eq(email))
            .all(&self.state.db_conn)
            .await?;
        match qr.len() {
            0 => Ok(None),
            _ => Ok(Some(qr)),
        }
    }
    pub async fn create_redis_blacklist(&self, header: RedisSchemaHeader) -> Result<(), GlobalError> {
        let mut schema = BlackList::new(header);
        schema.set_status(true).flush().await?;
        Ok(()) 
    }
    pub async fn sanitize_auth(&self, header: RedisSchemaHeader) -> Result<(), GlobalError> {
        let mut schema = TokenPair::new(header);
        schema.get_refresh_token().await?;
        if schema.refresh_token.is_some() {
            let mut refresh_schema = Refresh::new(RedisSchemaHeader {
                key: schema.refresh_token.clone().unwrap(),
                expire_at: None,
                con: self.state.redis_conn.clone(),
            });
            refresh_schema.del_all().await?;
        }
        schema.del_all().await?;
        Ok(()) 
    }
}
