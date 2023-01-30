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
            .values_panic(["kim@naver.com".into(),"a".into(),"kim".into()])
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
                    .col(ColumnDef::new(Docorg::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Docorg::UpdatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    //0: private, 1: public, 2: pending, 3: deleted
                    .col(ColumnDef::new(Docorg::Status).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Docorg::Table, Docorg::DocuserId)
                        .to(Docuser::Table, Docuser::Id))
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(Docorg::Table)
            .columns([Docorg::DocuserId, Docorg::Raw, Docorg::Status])
            .values_panic([1.into(), "hello world. this is new document".into(), 1.into()])
            .or_default_values()
            .to_owned();
        manager.exec_stmt(insert).await?;

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
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scope::DocuserId).integer().not_null())
                    .col(ColumnDef::new(Scope::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Scope::Table, Scope::DocuserId)
                        .to(Docuser::Table, Docuser::Id))
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(Scope::Table)
            .columns([Scope::DocuserId, Scope::Name])
            .values_panic([1.into(), "navydocuments".into()])
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
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DocorgScope::DocorgId).integer().not_null())
                    .col(ColumnDef::new(DocorgScope::ScopeId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgScope::Table, DocorgScope::DocorgId)
                        .to(Docorg::Table, Docorg::Id))
                    .foreign_key(
                        ForeignKey::create()
                        .from(DocorgScope::Table, DocorgScope::ScopeId)
                        .to(Scope::Table, Scope::Id))
                    .to_owned(),
            )
            .await?;

        let insert = Query::insert()
            .into_table(DocorgScope::Table)
            .columns([DocorgScope::DocorgId, DocorgScope::ScopeId])
            .values_panic([1.into(), 1.into()])
            .or_default_values()
            .to_owned();
        manager.exec_stmt(insert).await?;

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
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Scope::DocuserId).integer().not_null())
                    .col(ColumnDef::new(Scope::Name).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .from(Scope::Table, Scope::DocuserId)
                        .to(Docuser::Table, Docuser::Id))
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
            .drop_table(Table::drop().table(DocorgScope::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Docorg::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scope::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Docuser::Table).to_owned())
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
enum Image {
    Table,
    Id,
    Uri,
    //sync with docuorg
    Status,
}
