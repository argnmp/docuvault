use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use jsonwebtoken::{encode, Header};
use sea_orm::{entity:: *, query::*, DbErr};
use crate::{entity, AppState, routes::error::GlobalError, db::schema::redis::{RedisSchemaHeader, BlackList, TokenPair, Refresh}};

use super::{object::{Claims, ACCESS_KEYS, REFRESH_KEYS}, error::AuthError, constant::constant::{REFRESH_TOKEN_DUR, ACCESS_TOKEN_DUR}};

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
    pub async fn get_tokenpair_refresh_token(&self, access_token: &str) -> Result<Option<String>, GlobalError>{
        let mut schema = TokenPair::new(RedisSchemaHeader { 
            key: access_token.to_string(),
            expire_at: None,
            con: self.state.redis_conn.clone(), 
        });  
        schema.get_refresh_token().await?;
        return Ok(schema.refresh_token);
    }
    pub async fn get_tokenpair_refresh_token_with_deletion(&self, access_token: &str) -> Result<Option<String>, GlobalError>{
        let mut schema = TokenPair::new(RedisSchemaHeader { 
            key: access_token.to_string(),
            expire_at: None,
            con: self.state.redis_conn.clone(), 
        });  
        schema.get_refresh_token().await?;
        schema.del_all().await?;
        return Ok(schema.refresh_token);
    }
    pub async fn set_tokenpair(&self, access_token: &str, refresh_token: &str) -> Result<(), GlobalError>{

        let mut schema = TokenPair::new(RedisSchemaHeader {
            key: access_token.to_string(),
            expire_at: Some((chrono::Utc::now() + *ACCESS_TOKEN_DUR).timestamp() as usize),
            con: self.state.redis_conn.clone(),
        });
        schema.set_refresh_token(refresh_token.to_string()).flush().await?;

        return Ok(());
    }
    pub async fn get_refresh_ip(&self, refresh_token: &str) -> Result<Option<String>, GlobalError>{
        let mut refresh_schema = Refresh::new(RedisSchemaHeader {
            key: refresh_token.to_string(),
            expire_at: None,
            con: self.state.redis_conn.clone(),
        });
        refresh_schema.get_ip().await?;
        return Ok(refresh_schema.ip);
    }
    pub async fn set_refresh(&self, refresh_token: &str, ip: String) -> Result<(), GlobalError>{
        let mut schema = Refresh::new(RedisSchemaHeader {
            key: refresh_token.to_string(),
            expire_at: Some((chrono::Utc::now() + *REFRESH_TOKEN_DUR).timestamp() as usize),
            con: self.state.redis_conn.clone(),
        });
        schema.set_ip(ip.to_string()).flush().await?;

        return Ok(());
    }

    pub async fn remove_refresh_record(&self, refresh_token: &str) -> Result<(), GlobalError>{
    let mut refresh_schema = Refresh::new(RedisSchemaHeader {
        key: refresh_token.to_string(),
        expire_at: None,
        con: self.state.redis_conn.clone(),
    });
    refresh_schema.del_all().await?;
    return Ok(());
    }
    pub async fn set_redis_blacklist(&self, token: &str, expire_at: usize) -> Result<(), GlobalError> {
        let mut schema = BlackList::new(RedisSchemaHeader {
            key: token.to_string(),
            expire_at: Some(expire_at), 
            con: self.state.redis_conn.clone(),
        });
        schema.set_status(true).flush().await?;
        Ok(()) 
    }
    pub async fn disable_auth(&self, access_token: &str) -> Result<(), GlobalError> {
        let refresh_token = self.get_tokenpair_refresh_token_with_deletion(access_token).await?;
        if refresh_token.is_some() {
            let mut refresh_schema = Refresh::new(RedisSchemaHeader {
                key: refresh_token.unwrap(),
                expire_at: None,
                con: self.state.redis_conn.clone(),
            });
            refresh_schema.del_all().await?;
        }
        Ok(()) 
    }

    pub async fn issue_access_token(&self, user_id: i32) -> Result<String, GlobalError> {

        let claims = Claims {
            iat: chrono::Utc::now().timestamp(),
            exp: (chrono::Utc::now() + *ACCESS_TOKEN_DUR).timestamp(),
            iss: "docuvault".to_owned(),
            user_id,
            token_typ: "access".to_owned(),
        };     

        let access_token = encode(&Header::default(), &claims, &ACCESS_KEYS.encoding).map_err(|err|AuthError::from(err))?;

        return Ok(access_token);
    }
    pub async fn issue_refresh_token(&self, user_id: i32) -> Result<String, GlobalError> {

        let refresh_claims = Claims {
            iat: chrono::Utc::now().timestamp(),
            exp: (chrono::Utc::now() + *REFRESH_TOKEN_DUR).timestamp(),
            iss: "docuvault".to_owned(),
            user_id,
            token_typ: "refresh".to_owned(),
        };
        let refresh_token = encode(&Header::default(), &refresh_claims, &REFRESH_KEYS.encoding).map_err(|err|AuthError::from(err))?;

        return Ok(refresh_token);
    }
}
