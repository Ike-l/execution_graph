
#[derive(PartialEq, Eq)]
pub struct Link<Identifier, Priority> {
    pub from: Identifier,
    pub to: Identifier,
    priority: Priority,
}

impl<T: PartialEq + Eq, P: Ord> Ord for Link<T, P> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl<T: PartialEq + Eq, P: PartialOrd> PartialOrd for Link<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}