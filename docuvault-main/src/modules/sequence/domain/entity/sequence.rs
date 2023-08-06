pub struct Sequence {
    pub id: i32,
    pub uid: i32,
    pub title: String,
    pub scope_ids: Vec<i32>,
}
impl Sequence {
    pub fn new(id: i32, uid: i32, title: String, scope_ids: Vec<i32>) -> Self {
        Self {
            id, uid, title, scope_ids
        }
    }
}

pub struct SequenceObj {
    pub uid: i32,
    pub title: String,
    pub scope_ids: Vec<i32>,
}
impl SequenceObj {
    pub fn new(uid: i32, title: String, scope_ids: Vec<i32>) -> Self {
        Self {
            uid, title, scope_ids
        }
    }
}
