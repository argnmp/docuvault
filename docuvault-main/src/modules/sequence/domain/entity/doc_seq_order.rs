#[derive(Debug)]
pub struct DocSeqOrder {
    pub doc_id: i32,
    pub seq_id: i32,
    pub order: i32,
}
impl DocSeqOrder {
    pub fn new(doc_id: i32, seq_id: i32, order: i32) -> Self {
        Self { doc_id, seq_id, order }
    }
}
