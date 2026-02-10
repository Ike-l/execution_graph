use std::{rc::Rc, sync::atomic::{AtomicUsize, Ordering}};

pub struct Node<T> {
    // ready when 0
    ready: AtomicUsize,

    pub data: T,

    // Self or index to container
    out_neighbours: Vec<Rc<Self>>,
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            ready: AtomicUsize::new(0),
            data,
            out_neighbours: Vec::new()
        }
    }

    pub fn make_unready(&self) {
        self.ready.fetch_add(1, Ordering::SeqCst);
    }

    pub fn make_ready(&self) {
        self.ready.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn complete(&self) {
        for neighbour in self.out_neighbours.iter() {
            neighbour.make_ready();
        }
    }
}

impl<
    T: PartialEq   
> Node<T> {
    pub fn contains_child<'a>(&'a self, child: &T, mut seen: Vec<&'a T>) -> bool {
        // Found
        if *child == self.data {
            return true;
        }

        // Cycle
        if seen.iter().any(|seen| **seen == self.data) {
            return false;
        }

        seen.push(&self.data);
        self.out_neighbours.iter().any(|neighbour| {
            neighbour.contains_child(child, seen.clone())
        })
    }
}