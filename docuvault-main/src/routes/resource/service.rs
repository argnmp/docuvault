use crate::AppState;

#[derive(Clone, Debug)]
pub struct ResourceService {
    state: AppState,
}
impl ResourceService {
    pub fn new(shared_state: AppState) -> Self {
        Self {
            state: shared_state.clone(),
        }
    }
}
