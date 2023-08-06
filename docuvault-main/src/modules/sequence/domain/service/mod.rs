use std::borrow::BorrowMut;

use super::entity::doc_seq_order::DocSeqOrder;

pub struct SequenceDomainService {}
impl SequenceDomainService {
    pub fn up(doc_id: i32, mut docs: Vec<DocSeqOrder>) -> Result<Vec<DocSeqOrder>, ()> {
        docs.sort_by_key(|doc| doc.order);
        if let Some(doc) = docs.last() {
            if(doc_id == doc.doc_id) {
                return Ok(docs);
            }
        }
        else {
            return Err(());
        }

        let mut target_idx: Option<usize> = None;
        for (idx, doc) in docs.iter_mut().enumerate() {
            *doc.order.borrow_mut() = idx as i32; 
            if(doc.doc_id == doc_id){
                *doc.order.borrow_mut() = (idx + 1) as i32;
                target_idx = Some(idx);
            }
        }
        match target_idx {
            Some(idx) => {
                if let Some(elem) = docs.get_mut(idx + 1) {
                    *elem.order.borrow_mut() = idx as i32;
                }
                return Ok(docs);
            },
            None => {
                return Err(()); 
            }
        }
    }
    pub fn down(doc_id: i32, mut docs: Vec<DocSeqOrder>) -> Result<Vec<DocSeqOrder>, ()> {
        docs.sort_by_key(|doc| doc.order);
        if let Some(doc) = docs.get(0) {
            if(doc_id == doc.doc_id) {
                return Ok(docs);
            }
        }
        else {
            return Err(());
        }

        let mut target_idx: Option<usize> = None;
        for (idx, doc) in docs.iter_mut().enumerate() {
            *doc.order.borrow_mut() = idx as i32; 
            if(doc.doc_id == doc_id){
                *doc.order.borrow_mut() = (idx - 1) as i32;
                target_idx = Some(idx);
            }
        }
        match target_idx {
            Some(idx) => {
                if let Some(elem) = docs.get_mut(idx - 1) {
                    *elem.order.borrow_mut() = idx as i32;
                }
                return Ok(docs);
            },
            None => {
                return Err(()); 
            }
        }
    }

}

#[test]
fn seq_order_test() {
    let docs = vec![
        DocSeqOrder::new(1, 1, 5),
        DocSeqOrder::new(2, 1, 6),
        DocSeqOrder::new(3, 1, 3),
        DocSeqOrder::new(4, 1, 7),
    ];
    if let Ok(docs) = SequenceService::up(4, docs) {
        dbg!(docs); 
    } 
}
