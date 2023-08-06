use async_trait::async_trait;

use crate::{routes::error::GlobalError, modules::sequence::domain::entity::{sequence::Sequence, doc_seq_order::DocSeqOrder}};

// GlobalError을 axum에 의존적이지 않도록 만들 필요가 있다.
#[async_trait()]
pub trait SequenceUseCase {
    async fn get_seq(&self, seq_id: i32) -> Result<Sequence, GlobalError>;
    async fn get_docseqord(&self, seq: Sequence) -> Result<Vec<DocSeqOrder>, GlobalError>;
    async fn create_seq(&self, seq: Sequence) -> Result<(), GlobalError>;
    async fn remove_seq(&self, seq_id: i32) -> Result<(), GlobalError>;
    // async fn update(&self, seq: Sequence) -> Result<(), GlobalError>;
    async fn doc_alloc(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>;  
    async fn doc_dealloc(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>;  
    async fn doc_ord_up(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>;
    async fn doc_ord_down(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>;
}
