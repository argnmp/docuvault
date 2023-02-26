use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Docuser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Docuser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Docuser::Email).string().not_null())
                    .col(ColumnDef::new(Docuser::Hash).string().not_null())
                    .col(ColumnDef::new(Docuser::Nickname).string().not_null())
                    .col(ColumnDef::new(Docuser::CreatedAt).date_time().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Docuser::UpdatedAt).date_time().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .to_owned()
            )
            .await?;
        let insert = Query::insert()
            .into_table(Docuser::Table)
            .columns([Docuser::Email, Docuser::Hash, Docuser::Nickname])
            .values_panic(["abc@abc.com".into(),"$argon2id$v=19$m=4096,t=3,p=1$04bZG/BgZ88j2z6hwm+KPw$F+jgyuh+RxFgpZfAA+heTAdsCyDjU67rOFODgRNxgMo".into(),"abc".into()])
            .values_panic(["kim@kim.com".into(),"$argon2id$v=19$m=4096,t=3,p=1$04bZG/BgZ88j2z6hwm+KPw$F+jgyuh+RxFgpZfAA+heTAdsCyDjU67rOFODgRNxgMo".into(),"kim".into()])
            .or_default_values()
            .to_owned();
        manager.exec_stmt(insert).await?;

        manager
            .create_table(
                Table::create()
                    .table(Docorg::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Docorg::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Docorg::PrevId).integer())
                    .col(ColumnDef::new(Docorg::DocuserId).integer().not_null())
                    .col(ColumnDef::new(Docorg::Raw).string().not_null())
                    .col(ColumnDef::new(Docorg::Title).string().not_null())
                    .col(ColumnDef::new(Docorg::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Docorg::UpdatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    //0: private, 1: public, 2: pending, 3: deleted
                    .col(ColumnDef::new(Docorg::Status).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Docorg::Table, Docorg::DocuserId)
                        .to(Docuser::Table, Docuser::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        // let insert = Query::insert()
            // .into_table(Docorg::Table)
            // .columns([Docorg::DocuserId, Docorg::Title, Docorg::Raw, Docorg::Status])
            // .values_panic([1.into(),"First document from user 1".into(), "hello world. this is new document".into(), 1.into()])
            // .values_panic([1.into(),"Second document from user 1".into(), "You must have chaos within you to give birth to a dancing star.".into(), 1.into()])
            // .values_panic([1.into(),"Third document from user 1".into(), "That which does not kill us makes us stronger.".into(), 1.into()])
            // .values_panic([2.into(),"First document from user 2".into(), "hello world. this is new document".into(), 1.into()])
            // .values_panic([2.into(),"Second document from user 2".into(), "You must have chaos within you to give birth to a dancing star.".into(), 1.into()])
            // .values_panic([2.into(),"Third document from user 2".into(), "That which does not kill us makes us stronger.".into(), 1.into()])
            // .or_default_values()
            // .to_owned();
        // manager.exec_stmt(insert).await?;

        manager
            .create_table(
                Table::create()
                    .table(Scope::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Scope::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(Scope::DocuserId).integer().not_null())
                    .col(ColumnDef::new(Scope::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Scope::Table, Scope::DocuserId)
                        .to(Docuser::Table, Docuser::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(Scope::Table)
            .columns([Scope::DocuserId, Scope::Name])
            .values_panic([1.into(), "navydocuments".into()])
            .values_panic([1.into(), "kimtahencom".into()])
            .values_panic([2.into(), "google".into()])
            .values_panic([2.into(), "naver".into()])
            .values_panic([2.into(), "thvapour".into()])
            .or_default_values()
            .to_owned();
        manager.exec_stmt(insert).await?;


        manager
            .create_table(
                Table::create()
                    .table(DocorgScope::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DocorgScope::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                    )
                    .col(ColumnDef::new(DocorgScope::DocorgId).integer().not_null())
                    .col(ColumnDef::new(DocorgScope::ScopeId).integer().not_null())
                    .primary_key(Index::create().col(DocorgScope::DocorgId).col(DocorgScope::ScopeId))
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgScope::Table, DocorgScope::DocorgId)
                        .to(Docorg::Table, Docorg::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgScope::Table, DocorgScope::ScopeId)
                        .to(Scope::Table, Scope::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        // let insert = Query::insert()
            // .into_table(DocorgScope::Table)
            // .columns([DocorgScope::DocorgId, DocorgScope::ScopeId])
            // .values_panic([1.into(), 1.into()])
            // .values_panic([1.into(), 2.into()])
            // .values_panic([2.into(), 1.into()])
            // .values_panic([3.into(), 2.into()])
            // .or_default_values()
            // .to_owned();
        // manager.exec_stmt(insert).await?;

        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Tag::Value).string().not_null())
                    .to_owned(),
            )
            .await?;

        // let insert = Query::insert()
            // .into_table(Tag::Table)
            // .columns([Tag::Value])
            // .values_panic(["cpp".into()])
            // .values_panic(["rust".into()])
            // .values_panic(["javascript".into()])
            // .values_panic(["algorithm".into()])
            // .or_default_values()
            // .to_owned();
        // manager.exec_stmt(insert).await?;

        manager
            .create_table(
                Table::create()
                    .table(DocorgTag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DocorgTag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                    )
                    .col(ColumnDef::new(DocorgTag::DocorgId).integer().not_null())
                    .col(ColumnDef::new(DocorgTag::TagId).integer().not_null())
                    .primary_key(Index::create().col(DocorgTag::DocorgId).col(DocorgTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgTag::Table, DocorgTag::DocorgId)
                        .to(Docorg::Table, Docorg::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgTag::Table, DocorgTag::TagId)
                        .to(Tag::Table, Tag::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;


        // let insert = Query::insert()
            // .into_table(DocorgTag::Table)
            // .columns([DocorgTag::DocorgId, DocorgTag::TagId])
            // .values_panic([1.into(), 1.into()])
            // .values_panic([1.into(), 2.into()])
            // .values_panic([1.into(), 3.into()])
            // .values_panic([1.into(), 4.into()])
            // .values_panic([2.into(), 2.into()])
            // .values_panic([2.into(), 4.into()])
            // .values_panic([3.into(), 1.into()])
            // .values_panic([3.into(), 3.into()])
            // .or_default_values()
            // .to_owned();
        // manager.exec_stmt(insert).await?;

        manager
            .create_table(
                Table::create()
                    .table(Convert::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Convert::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                    )
                    .col(ColumnDef::new(Convert::DocorgId).integer().not_null())
                    // there is no data when status is pending
                    .col(ColumnDef::new(Convert::Data).string())
                    .col(ColumnDef::new(convert::CTypeEnum.into_iden()).integer().not_null())
                    .col(ColumnDef::new(convert::StatusEnum.into_iden()).integer().not_null())
                    .primary_key(Index::create().col(Convert::DocorgId).col(convert::CTypeEnum.into_iden()))
                    .foreign_key(
                        ForeignKey::create()
                        .from(Convert::Table, Convert::DocorgId)
                        .to(Docorg::Table, Docorg::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sequence::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sequence::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(Sequence::Title).string().not_null())
                    .col(ColumnDef::new(Sequence::DocuserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Sequence::Table, Sequence::DocuserId)
                        .to(Docuser::Table, Docuser::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DocorgSequence::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DocorgSequence::SequenceId).integer().not_null())
                    .col(ColumnDef::new(DocorgSequence::DocorgId).integer().not_null())
                    .col(ColumnDef::new(DocorgSequence::Order).integer().not_null())
                    .primary_key(Index::create().col(DocorgSequence::SequenceId).col(DocorgSequence::DocorgId))
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgSequence::Table, DocorgSequence::DocorgId)
                        .to(Docorg::Table, Docorg::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgSequence::Table, DocorgSequence::SequenceId)
                        .to(Sequence::Table, Sequence::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ScopeSequence::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ScopeSequence::SequenceId).integer().not_null())
                    .col(ColumnDef::new(ScopeSequence::ScopeId).integer().not_null())
                    .primary_key(Index::create().col(ScopeSequence::SequenceId).col(ScopeSequence::ScopeId))
                    .foreign_key(
                        ForeignKey::create()
                        .from(ScopeSequence::Table, ScopeSequence::SequenceId)
                        .to(Sequence::Table, Sequence::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .foreign_key(
                        ForeignKey::create()
                        .from(ScopeSequence::Table, ScopeSequence::ScopeId)
                        .to(Scope::Table, Scope::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Image::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Image::Uri).string().not_null())
                    //0: private, 1: public, 2: pending, 3: deleted
                    .col(ColumnDef::new(Image::Status).integer().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ScopeSequence::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DocorgSequence::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sequence::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Convert::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DocorgTag::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DocorgScope::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Docorg::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scope::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Image::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Docuser::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Docorg {
    Table,
    Id,
    PrevId,
    Title,
    DocuserId,
    Raw,
    CreatedAt,
    UpdatedAt,
    Status, 
}

#[derive(Iden)]
enum Docuser {
    Table,
    Id,
    Email,
    Hash,
    Nickname,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Scope {
    Table,
    Id,
    DocuserId,
    Name,
}
#[derive(Iden)]
enum DocorgScope {
    Table,
    Id,
    DocorgId,
    ScopeId, 
}

#[derive(Iden)]
enum Tag {
    Table,
    Id,
    Value,
}
#[derive(Iden)]
enum DocorgTag {
    Table,
    Id,
    DocorgId,
    TagId,
}
#[derive(Iden)]
enum Convert{
    Table,
    Id,
    DocorgId,
    Data,
}
mod convert {
    use sea_orm_migration::prelude::*;
    use sea_orm_migration::sea_orm::{EnumIter, DeriveActiveEnum};

    // can be html, hwp, docx ...
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
    pub enum CType {
        Html = 0,
        Txt = 1,
        Docx = 2,
        Hwp = 3,
    }
    // can be data it self if html or file location if hwp or docs ..
#[derive(EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
    pub enum Status {
        Pending = 0,
        Pass = 1,
        Fail = 2,
    }

}


#[derive(Iden)]
enum Sequence {
    Table,
    Id,
    DocuserId,
    Title,
}

#[derive(Iden)]
enum DocorgSequence {
    Table,
    SequenceId,
    DocorgId,
    Order,
}

#[derive(Iden)]
enum ScopeSequence {
    Table,
    SequenceId,
    ScopeId,
}

#[derive(Iden)]
enum Image {
    Table,
    Id,
    Uri,
    //sync with docuorg
    Status,
}
