
#[derive(Debug)]
pub struct Link<Identifier> {
    pub from: Identifier,
    pub to: Identifier,
}

impl<T> Link<T> {
    pub fn new(from: T, to: T) -> Self {
        Self { from, to }
    }
}
