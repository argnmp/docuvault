use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListPayload{
    pub scope_id: Vec<i32>,
    pub unit_size: Option<u64>,
    pub unit_number: Option<u64>,
}
