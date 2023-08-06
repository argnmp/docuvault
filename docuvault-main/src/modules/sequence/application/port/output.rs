use async_trait::async_trait;

use crate::{routes::error::GlobalError, modules::sequence::domain::entity::{sequence::{Sequence, SequenceObj}, doc_seq_order::DocSeqOrder}};

#[async_trait()]
pub trait SequenceRepositoryPort: std::fmt::Debug {
    async fn load_seq(&self, seq_id: i32) -> Result<Sequence, GlobalError>;
    async fn save_seq(&self, seq: Sequence) -> Result<(), GlobalError>;
    async fn create_seq(&self, seq: SequenceObj) -> Result<(), GlobalError>;
    async fn delete_seq(&self, seq_id: i32) -> Result<(), GlobalError>;
    async fn load_docseqord(&self, seq_id: i32) -> Result<Vec<DocSeqOrder>, GlobalError>;
    async fn save_docseqord(&self, seq_id: i32, docseqord: Vec<DocSeqOrder>) -> Result<(), GlobalError>;
}

