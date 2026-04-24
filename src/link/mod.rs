use crate::prelude::Flow;

#[derive(Debug, Clone)]
pub struct Link<Identifier> {
    pub from: Identifier,
    pub to: Identifier,
    pub flow: Flow
}

impl<T> Link<T> {
    pub fn free(from: T, to: T) -> Self {
        Self { from, to, flow: Flow::Free }
    }

    pub fn new(from: T, to: T, flow: Flow) -> Self {
        Self { from, to, flow }
    }
}
