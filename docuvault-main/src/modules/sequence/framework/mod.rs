use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect, JoinType, RelationTrait, FromQueryResult, DbErr};
use sea_orm::{entity::*, query::*};
use serde::Serialize;

use crate::{routes::error::GlobalError, entity};

use super::domain::entity::sequence::SequenceObj;
use super::{application::port::output::SequenceRepositoryPort, domain::entity::{sequence::Sequence, doc_seq_order::DocSeqOrder}, error::SequenceError};

#[derive(Debug)]
pub struct SequencePersistentAdapter {
    conn: DatabaseConnection,
}
impl SequencePersistentAdapter {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
}

#[async_trait()]
impl SequenceRepositoryPort for SequencePersistentAdapter {
    async fn load_seq(&self, seq_id: i32) -> Result<Sequence, GlobalError>{

        #[derive(FromQueryResult, Serialize, Debug)]
        struct Wrapper {
            id: i32,
            title: String,
            docuser_id: i32,
            scope_id: i32,
        };

        let seq_res = entity::sequence::Entity::find_by_id(seq_id)
            .join_rev(JoinType::LeftJoin, entity::scope_sequence::Relation::Sequence.def())
            .column_as(entity::scope_sequence::Column::ScopeId, "scope_id")
            .into_model::<Wrapper>()
            .all(&self.conn)
            .await?;
        
        match seq_res.len() {
            0 => {
                return Err(SequenceError::SequenceNotExist.into())
            },
            _ => {
                let scope_ids = seq_res.iter().map(|seq| seq.scope_id).collect::<Vec<i32>>();
                return Ok(Sequence::new(seq_res[0].id, seq_res[0].docuser_id, seq_res[0].title.clone(), scope_ids));
            }
        };  
    }

    async fn save_seq(&self, seq: Sequence) -> Result<(), GlobalError>{
        &self.conn.transaction::<_, (), DbErr>(|txn|{
            Box::pin(async move {
                
                let sequence = entity::sequence::Entity::find_by_id(seq.id)
                    .one(txn)
                    .await?;
                match sequence {
                    Some(target) => {
                        let mut t = target.into_active_model();
                        t.title = Set(seq.title);
                        t.update(txn).await?;
                    }, 
                    None => {
                    } 
                }
                
                entity::scope_sequence::Entity::delete_many()
                    .filter(entity::scope_sequence::Column::SequenceId.eq(seq.id))
                    .exec(txn)
                    .await?;

                let records = seq.scope_ids.iter().map(|scope_id|{
                    entity::scope_sequence::ActiveModel {
                        sequence_id: Set(seq.id),
                        scope_id: Set(*scope_id),
                    }
                }).collect::<Vec<_>>();

                entity::scope_sequence::Entity::insert_many(records).exec(txn).await?;

                Ok(())
            })
        }).await?;

        Ok(())
    }
    async fn create_seq(&self, seq: SequenceObj) -> Result<(), GlobalError> {
        &self.conn.transaction::<_, (), DbErr>(|txn|{
            Box::pin(async move {
                let model = entity::sequence::ActiveModel {
                    title: Set(seq.title),
                    docuser_id: Set(seq.uid),
                    ..Default::default()
                };
                let res = entity::sequence::Entity::insert(model).exec(txn).await?;

                let records = seq.scope_ids.iter().map(|scope_id|{
                    entity::scope_sequence::ActiveModel {
                        sequence_id: Set(res.last_insert_id),
                        scope_id: Set(*scope_id),
                    }
                }).collect::<Vec<_>>();

                entity::scope_sequence::Entity::insert_many(records).exec(txn).await?;

                Ok(())
            })
        }).await?;
        Ok(())
        
    }
    async fn delete_seq(&self, seq_id: i32) -> Result<(), GlobalError> {
        entity::sequence::Entity::delete_by_id(seq_id)
            .exec(&self.conn)
            .await?;
        Ok(())
    }
    

    async fn load_docseqord(&self, seq_id: i32) -> Result<Vec<DocSeqOrder>, GlobalError>{
        #[derive(FromQueryResult, Serialize, Debug)]
        struct Wrapper {
            id: i32,
            title: String,
            docuser_id: i32,
            scope_id: i32,
        };
        let res = entity::docorg_sequence::Entity::find()
            .filter(entity::docorg_sequence::Column::SequenceId.eq(seq_id))
            .all(&self.conn)
            .await?;

        let docseqorder = res.into_iter().map(|rec| { DocSeqOrder::new(rec.docorg_id, rec.sequence_id, rec.order) }).collect::<Vec<DocSeqOrder>>();
        return Ok(docseqorder);
    }
    async fn save_docseqord(&self, seq_id: i32, docseqord: Vec<DocSeqOrder>) -> Result<(), GlobalError>{
        &self.conn.transaction::<_, (), DbErr>(|txn|{
            Box::pin(async move {
                entity::docorg_sequence::Entity::delete_many()
                    .filter(entity::docorg_sequence::Column::SequenceId.eq(seq_id))
                    .exec(txn)
                    .await?;

                let records = docseqord.iter().map(|docseqorder|{
                    entity::docorg_sequence::ActiveModel {
                        sequence_id: Set(docseqorder.seq_id),
                        docorg_id: Set(docseqorder.doc_id),
                        order: Set(docseqorder.order)
                    }
                }).collect::<Vec<_>>();

                entity::docorg_sequence::Entity::insert_many(records).exec(txn).await?;

                Ok(())
            })
        }).await?;

        Ok(())
    }
}

