use std::collections::BTreeSet;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm::{entity::*, query::*, FromQueryResult};
use crate::entity;
use crate::routes::error::GlobalError;

use super::domain::entity::tag::Tag;
use super::{application::port::output::TagSetRepositoryPort, domain::entity::tag_set::TagSet};

#[derive(Debug)]
pub struct TagSetRepositoryAdapter {
    conn: DatabaseConnection, 
}
impl TagSetRepositoryAdapter {
    pub fn new(conn: DatabaseConnection) -> Self {
        TagSetRepositoryAdapter { conn }
    }
}
#[async_trait()]
impl TagSetRepositoryPort for TagSetRepositoryAdapter {

    async fn load(&self) -> Result<TagSet, GlobalError> {
        let tags = entity::tag::Entity::find()
            .order_by_asc(entity::tag::Column::Value)
            .all(&self.conn)
            .await?;
        let tag_set = tags.into_iter().map(|tag| Tag::new(tag.value)).collect::<BTreeSet<Tag>>();
        Ok(TagSet::new(tag_set)) 
    }
    async fn save(&self, tag_set: &TagSet) -> Result<(), GlobalError> {
        let old_tags = self.load().await?;
        let new_tags = tag_set.tags.iter().filter(|tag| !old_tags.tags.contains(tag)).map(|tag| tag).collect::<Vec<&Tag>>();
        let records = new_tags.iter().map(|tag| entity::tag::ActiveModel{
            value: Set(tag.value.clone()),
            ..Default::default()
        }).collect::<Vec<_>>();
        let res = entity::tag::Entity::insert_many(records).exec(&self.conn).await?; 
        Ok(())
    }
}
