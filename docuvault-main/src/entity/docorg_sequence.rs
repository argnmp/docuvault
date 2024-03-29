//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "docorg_sequence")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub sequence_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub docorg_id: i32,
    pub order: i32,
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
    #[sea_orm(
        belongs_to = "super::sequence::Entity",
        from = "Column::SequenceId",
        to = "super::sequence::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Sequence,
}

impl Related<super::docorg::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Docorg.def()
    }
}

impl Related<super::sequence::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sequence.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
