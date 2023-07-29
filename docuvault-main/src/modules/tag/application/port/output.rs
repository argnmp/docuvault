use std::collections::BTreeSet;

use async_trait::async_trait;

use crate::{modules::tag::domain::entity::tag_set::TagSet, routes::error::GlobalError};

#[async_trait()]
pub trait TagSetRepositoryPort: std::fmt::Debug {
    async fn load(&self) -> Result<TagSet, GlobalError>; 
    async fn save(&self, tag_set: &TagSet) -> Result<(), GlobalError>;
}
