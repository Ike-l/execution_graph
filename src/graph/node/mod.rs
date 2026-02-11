use std::{fmt::Debug, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};

use tracing::{Level, event, span};

use crate::prelude::sync::{RwLock, Arc};

pub struct Node<T> {
    // ready when 0
    ready: AtomicUsize,

    // status
    completed: AtomicBool,

    data: T,

    // Self or index to container
    out_neighbourhood: Vec<Arc<RwLock<Self>>>,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{:?}, ready: {:?}, completed: {:?}, neighbourhood degree: {:?}", 
                self.data, 
                self.ready, 
                self.completed, 
                self.out_neighbourhood.len()
        )
    }
}

impl<
    T
> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            ready: AtomicUsize::new(0),
            completed: AtomicBool::new(false),
            data,
            out_neighbourhood: Vec::new()
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn insert_out_neighbour(&mut self, neighbour: Arc<RwLock<Self>>) {
        self.out_neighbourhood.push(neighbour);
    }

    pub fn make_unready(&self) {
        self.ready.fetch_add(1, Ordering::AcqRel);
    }

    pub fn make_ready(&self) {
        self.ready.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) == 0 && !self.completed.load(Ordering::Acquire)
    }

    pub fn complete(&self) {
        let result = self.completed.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed);
        assert!(result.is_ok());

        for neighbour in self.out_neighbourhood.iter() {
            neighbour.read().unwrap().make_ready();
        }
    }
}

impl<
    T: PartialEq + Debug
> Node<T> {
    pub fn contains_child<'a>(&'a self, child: &T, mut seen: Vec<&'a T>) -> bool {
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
            let Ok(neighbour) = neighbour.try_read() else { 
                event!(Level::TRACE, "Failed to get guard");
                return false 
            };

            let span = span!(Level::TRACE, "Checking neighbour", data =? neighbour.data());
            let _enter = span.enter();
            
            neighbour.contains_child(child, seen.clone())
        })
    }
}