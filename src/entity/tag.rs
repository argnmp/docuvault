//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::docorg_tag::Entity")]
    DocorgTag,
}

impl Related<super::docorg_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DocorgTag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}