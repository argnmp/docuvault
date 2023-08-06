use async_trait::async_trait;

use crate::{modules::sequence::domain::{entity::{sequence::{Sequence, SequenceObj}, doc_seq_order::DocSeqOrder}, service::SequenceDomainService}, routes::error::GlobalError};

use super::port::{output::SequenceRepositoryPort, input::SequenceUseCase};

#[derive(Debug)]
pub struct SequenceService {
    sequence_persistent_port: Box<dyn SequenceRepositoryPort + Send + Sync>,
}
impl SequenceService {
    pub fn new(sequence_persistent_port: Box<dyn SequenceRepositoryPort + Send + Sync>) -> Self {
        Self {
            sequence_persistent_port,
        }
    }
}

#[async_trait()]
impl SequenceUseCase for SequenceService {
    // sequence 에러로 별도 처리가 필요!!
    async fn get_seq(&self, seq_id: i32) -> Result<Sequence, GlobalError>{
        Ok(self.sequence_persistent_port.load_seq(seq_id).await?)
    }
    async fn get_docseqord(&self, seq: Sequence) -> Result<Vec<DocSeqOrder>, GlobalError>{
        Ok(self.sequence_persistent_port.load_docseqord(seq.id).await?)
    }
    async fn create_seq(&self, seq: SequenceObj) -> Result<(), GlobalError>{
        self.sequence_persistent_port.create_seq(seq).await?;
        Ok(())
    }
    async fn remove_seq(&self, seq_id: i32) -> Result<(), GlobalError>{
        self.sequence_persistent_port.delete_seq(seq_id).await?;
        Ok(())
    }
    async fn doc_alloc(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>{
        let mut order = self.sequence_persistent_port.load_docseqord(seq.id).await?;
        order.push(DocSeqOrder::new(doc_id, seq.id, order.len() as i32));
        self.sequence_persistent_port.save_docseqord(seq.id, order);
        Ok(())
    }
    async fn doc_dealloc(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>{
        let mut order = self.sequence_persistent_port.load_docseqord(seq.id).await?;
        let order = order.into_iter().filter(|ord| ord.doc_id != doc_id).collect::<Vec<DocSeqOrder>>();
        self.sequence_persistent_port.save_docseqord(seq.id, order);
        Ok(())
    }
    async fn doc_ord_up(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>{
        let order = self.sequence_persistent_port.load_docseqord(seq.id).await?;

        // error 처리 필요
        let order = SequenceDomainService::up(doc_id, order).unwrap();
        self.sequence_persistent_port.save_docseqord(seq.id, order).await?; 
        Ok(())
    }
    async fn doc_ord_down(&self, seq: Sequence, doc_id: i32) -> Result<(), GlobalError>{
        let order = self.sequence_persistent_port.load_docseqord(seq.id).await?;

        // error 처리 필요
        let order = SequenceDomainService::down(doc_id, order).unwrap();
        self.sequence_persistent_port.save_docseqord(seq.id, order).await?; 
        Ok(())
    }
}
