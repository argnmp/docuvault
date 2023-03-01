use std::collections::BTreeSet;

use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ScopeAllResponse {
    pub scopes: Vec<(i32, String)>
}
#[derive(Debug, Deserialize)]
pub struct TagPayload {
    pub scope_ids: Vec<i32>
}

#[derive(FromQueryResult, Serialize, Debug)]
pub struct Docs {
    pub id: i32,
    pub scope_id: i32,
    pub title: String,
    pub seq_id: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_id: Option<i32>,
}
#[derive(Serialize, Debug)]
pub struct CompDocs {
    pub id: i32,
    pub scope_ids: BTreeSet<i32>,
    pub seq_ids: BTreeSet<i32>,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_ids: BTreeSet<i32>,
}
#[derive(FromQueryResult, Serialize, Debug)]
pub struct SeqDocs {
    pub id: i32,
    pub scope_id: i32,
    pub seq_id: i32,
    pub seq_order: i32,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_id: Option<i32>,
}
#[derive(Serialize, Debug)]
pub struct SeqCompDocs {
    pub id: i32,
    pub scope_ids: BTreeSet<i32>,
    pub seq_id: i32,
    pub seq_order: i32,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_ids: BTreeSet<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListPayload{
    pub scope_ids: Vec<i32>,
    pub unit_size: Option<u64>,
    pub unit_number: Option<u64>,
    pub tag_id: Option<i32>,
}
#[derive(Debug, Deserialize)]
pub struct SequenceAllPayload{
    pub scope_ids: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SequenceListPayload{
    pub scope_ids: Vec<i32>,
    pub seq_id: i32, 
}
#[derive(Debug, Deserialize)]
pub struct SeqNewPayload{
    pub scope_ids: Vec<i32>,
    pub title: String, 
}
#[derive(Debug, Deserialize)]
pub struct SeqDeletePayload{
    pub seq_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SeqOutPayload {
    pub seq_id: i32,
    pub doc_id: i32,
}
#[derive(Debug, Deserialize)]
pub struct SeqInPayload {
    pub seq_id: i32,
    pub doc_id: i32,
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct SeqOrder {
    pub seq_order: i32,
    pub doc_id: i32,
}
#[derive(Debug, Deserialize)]
pub struct SeqUpdatePayload {
    pub seq_id: i32,
    pub order: Vec<SeqOrder>,
}
