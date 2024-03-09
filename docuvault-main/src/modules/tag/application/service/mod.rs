use async_trait::async_trait;
use sea_orm::DatabaseTransaction;

use crate::{routes::error::GlobalError, modules::tag::domain::entity::{tag::Tag, tag_set::TagSet}};

use super::port::{input::TagSetUseCase, output::TagSetRepositoryPort};

#[derive(Debug)]
pub struct TagSetService {
    tag_set_persistent_port: Box<dyn TagSetRepositoryPort + Send + Sync>,
    tag_set_memory_port: Box<dyn TagSetRepositoryPort + Send + Sync>
}
impl TagSetService {
    pub fn new(tag_set_persistent_port: Box<dyn TagSetRepositoryPort + Send + Sync>, tag_set_memory_port: Box<dyn TagSetRepositoryPort + Send + Sync>) -> Self {
        Self {
            tag_set_persistent_port,
            tag_set_memory_port
        }
    }
}

#[async_trait()]
impl TagSetUseCase for TagSetService {
    async fn get(&self, txn: &DatabaseTransaction) -> Result<TagSet, GlobalError> {
        let tag_set = self.tag_set_persistent_port.load(txn).await?;
        Ok(tag_set)
    }
    async fn add(&self, txn: &DatabaseTransaction, tag: String) -> Result<(), GlobalError> {
        let mut tag_set = self.tag_set_persistent_port.load(txn).await?; 
        tag_set.add_tag(Tag::new(tag));
        self.tag_set_persistent_port.save(txn, &tag_set).await?;
        self.tag_set_memory_port.save(txn, &tag_set).await?;
        Ok(())
    }
    async fn check_existance(&self, txn: &DatabaseTransaction, tag: String) -> Result<bool, GlobalError> {
        let mut tag_set = self.get(txn).await?; 
        Ok(tag_set.tags.contains(&Tag::new(tag)))
    }
}
