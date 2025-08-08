use getset::{Getters, MutGetters, Setters};
use std::collections::VecDeque;

#[derive(Debug, Clone, Default, Getters)]
pub(crate) struct ValqMsg {
    #[getset(get = "pub")]
    id: u64,
    #[getset(get = "pub")]
    body: String,
}

impl ValqMsg {
    pub(crate) fn new(id: u64, body: String) -> Self {
        Self { id, body }
    }
}

#[derive(Debug, Clone, Getters, Setters, MutGetters)]
pub(crate) struct ValqType {
    #[getset(get = "pub", set = "pub")]
    id_sequence: u64,
    #[getset(get = "pub", get_mut = "pub")]
    msgs: VecDeque<ValqMsg>,
}

impl ValqType {
    pub(crate) fn new() -> Self {
        Self {
            id_sequence: 1,
            msgs: VecDeque::new(),
        }
    }
}
