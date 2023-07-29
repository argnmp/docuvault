use std::collections::BTreeSet;

use super::tag::Tag;

pub struct TagSet{
    pub tags: BTreeSet<Tag>,
}
impl TagSet {
    pub fn new(tags: BTreeSet<Tag>) -> Self {
        TagSet {
            tags
        }
    }
    pub fn add_tag(&mut self, tag: Tag) -> &mut Self {
        self.tags.insert(tag); 
        self
    }
    pub fn remove_tag(&mut self, tag: Tag) -> &mut Self {
        self.tags.remove(&tag);
        self
    }
}
