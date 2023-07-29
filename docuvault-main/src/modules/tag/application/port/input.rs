use async_trait::async_trait;

use crate::{modules::tag::domain::entity::{tag::Tag, tag_set::TagSet}, routes::error::GlobalError};

#[async_trait()]
pub trait TagSetUseCase {
    async fn get(&self) -> Result<TagSet, GlobalError>;
    async fn add(&self, tag: String) -> Result<(), GlobalError>;    
    // async fn remove(tag: String) -> Result<(), GlobalError>;
    async fn check_existance(&self, tag: String) -> Result<bool, GlobalError>;
}
