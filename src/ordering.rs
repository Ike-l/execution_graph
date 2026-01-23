use std::collections::HashSet;

use crate::prelude::SystemId;

pub trait ExecutionOrdering {
    type Item;

    fn subsume(&self, superset: &HashSet<Self::Item>) -> Self;
    /// before | after | priority
    fn consume(self) -> ( HashSet<Self::Item>, HashSet<Self::Item>, f64 );
}

/// "Before": This node is "Before" everything in this HashSet<Id>
/// Not to be confused with "Before": Everything in this HashSet<Id> is before this node
#[derive(Debug, Default, Clone)]
pub struct SchedulerOrdering {
    before: HashSet<SystemId>,
    after: HashSet<SystemId>,
    priority: f64
}

impl SchedulerOrdering {
    pub fn consume(&mut self, other: Self) {
        self.before.extend(other.before);
        self.after.extend(other.after);
        self.set_priority(other.priority);
    }

    pub fn set_priority(&mut self, new_priority: f64) {
        self.priority = new_priority;
    }

    pub fn insert_before(mut self, system_id: SystemId) -> Self {
        self.before.insert(system_id);
        self
    }
    
    pub fn insert_after(mut self, system_id: SystemId) -> Self {
        self.after.insert(system_id);
        self
    }

    pub fn before(&self) -> &HashSet<SystemId> {
        &self.before
    }
    
    pub fn after(&self) -> &HashSet<SystemId> {
        &self.after
    }
}

impl ExecutionOrdering for SchedulerOrdering {
    type Item = SystemId;

    fn subsume(&self, superset: &HashSet<Self::Item>) -> Self {
        Self {
            before: self.before.intersection(superset).cloned().collect(),
            after: self.after.intersection(superset).cloned().collect(),
            priority: self.priority
        }
    }

    fn consume(self) -> ( HashSet<Self::Item>, HashSet<Self::Item>, f64 ) {
        ( self.before, self.after, self.priority )
    }
}