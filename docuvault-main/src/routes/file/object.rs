use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestParams {
    pub vote: i32,
}
