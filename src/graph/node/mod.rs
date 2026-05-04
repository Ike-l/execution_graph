use std::{collections::HashSet, fmt::Debug, hash::Hash, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

use parking_lot::RwLock;
use tracing::{Level, event, span};

use crate::prelude::Status;

pub mod status;

pub struct Node<T> {
    // ready when 0
    ready: AtomicUsize,

    status: Status,

    data: T,

    // Self or index to container
    out_neighbourhood: Vec<Arc<RwLock<Self>>>,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{:?}, ready: {:?}, status: {:?}, neighbourhood degree: {:?}", 
                self.data, 
                self.ready, 
                self.status, 
                self.out_neighbourhood.len()
        )
    }
}

impl<
    T: Debug
> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            ready: AtomicUsize::new(0),
            status: Status::Ready,
            data,
            out_neighbourhood: Vec::new()
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn out_degree(&self) -> usize {
        self.out_neighbourhood.len()
    }

    pub fn in_degree(&self) -> usize {
        self.ready.load(Ordering::Acquire)
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn insert_out_neighbour(&mut self, neighbour: Arc<RwLock<Self>>) {
        self.out_neighbourhood.push(neighbour);
    }

    pub fn make_one_unready(&self) {
        self.ready.fetch_add(1, Ordering::AcqRel);
    }

    pub fn make_one_ready(&self) {
        self.ready.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) == 0 && 
        matches!(self.status, Status::Ready)
    }

    pub fn complete(&mut self) {
        self.status = Status::Complete;

        let span = span!(Level::DEBUG, "Notifying Neighbours");
        let _enter = span.enter();

        for neighbour in self.out_neighbourhood.iter() {
            event!(Level::TRACE, neighbour =? neighbour.read().data());

            neighbour.read().make_one_ready();
        }
    }
}

impl<
    T: Debug + PartialEq + Eq + Hash + Clone
> Node<T> {
    pub fn contains_child(
        &self, 
        child: &T, 
        seen: &mut HashSet<T>
    ) -> bool {
        // Found
        if *child == self.data {
            event!(Level::TRACE, "Found");
            return true;
        }

        // Cycle
        if !seen.insert(self.data.clone()) {
            event!(Level::TRACE, "Cycle");
            return false;
        }

        self.out_neighbourhood.iter().any(|neighbour| {
            // Assuming the guard is held by the current function, this would indicate a cycle
            let Some(neighbour) = neighbour.try_read() else { 
                event!(Level::TRACE, "Failed to get guard");
                return false 
            };

            let span = span!(Level::TRACE, "Checking neighbour", data =? neighbour.data());
            let _enter = span.enter();
            
            neighbour.contains_child(child, seen)
        })
    }
}

impl<T> Node<T> where T: Debug + PartialEq {
    /// If you know the length if small
    pub fn contains_child_2<'a>(
        &'a self, 
        child: &T, 
        mut seen: Vec<&'a T>
    ) -> bool {
        // Found
        if *child == self.data {
            event!(Level::TRACE, "Found");
            return true;
        }

        // Cycle
        if seen.iter().any(|seen| **seen == self.data) {
            event!(Level::TRACE, "Cycle");
            return false;
        }

        seen.push(&self.data);
        self.out_neighbourhood.iter().any(|neighbour| {
            // Assuming the guard is held by the current function, this would indicate a cycle
            let Some(neighbour) = neighbour.try_read() else { 
                event!(Level::TRACE, "Failed to get guard");
                return false 
            };

            let span = span!(Level::TRACE, "Checking neighbour", data =? neighbour.data());
            let _enter = span.enter();
            
            neighbour.contains_child_2(child, seen.clone())
        })
    }
}