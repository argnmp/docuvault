pub mod obj_hash {
    use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

    pub fn hash_to_limit<T: Hash>(n: u64, target: &T) -> usize{
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        (hasher.finish() % n) as usize
    }
}
