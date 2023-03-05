//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "docfile")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub docorg_id: Option<i32>,
    pub object_id: String,
    pub name: String,
    pub ftype: String,
    pub size: i64,
    pub uri: Option<String>,
    pub is_fixed: bool,
    pub status: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::docorg::Entity",
        from = "Column::DocorgId",
        to = "super::docorg::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Docorg,
}

impl Related<super::docorg::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Docorg.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
