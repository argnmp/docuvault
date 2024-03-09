use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

use crate::{modules::tag::domain::entity::{tag::Tag, tag_set::TagSet}, routes::error::GlobalError};

#[async_trait()]
pub trait TagSetUseCase {
    async fn get(&self, txn: &DatabaseTransaction) -> Result<TagSet, GlobalError>;
    async fn add(&self, txn: &DatabaseTransaction, tag: String) -> Result<(), GlobalError>;    
    // async fn remove(tag: String) -> Result<(), GlobalError>;
    async fn check_existance(&self, txn: &DatabaseTransaction, tag: String) -> Result<bool, GlobalError>;
}
