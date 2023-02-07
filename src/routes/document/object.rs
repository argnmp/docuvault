use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentPayload {
    pub document: String,
    pub tags: Vec<String>,
    pub scope_id: i32,
    pub prev_document_id: Option<i32>
}
