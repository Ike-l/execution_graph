pub enum Flow {
    Free,

    Custom(usize),
}

impl Into<Flow> for Flow {
    fn into(self) -> Flow {
        match self {
            Flow::Free => Self::Custom(0),
            Flow::Custom(_) => self,
        }
    }
}