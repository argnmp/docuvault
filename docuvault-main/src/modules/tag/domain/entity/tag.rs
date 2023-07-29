#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    pub value: String,
}
impl Tag {
    pub fn new(value: String) -> Self {
        Self {
            value,
        }
    }
}
