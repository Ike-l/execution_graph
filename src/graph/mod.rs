use std::{rc::Rc, sync::{Mutex, atomic::Ordering}};

use crate::{graph::node::Node, link::Link};

pub mod node;

pub struct Graph<T> {
    nodes: Vec<Rc<Node<T>>>
}

impl<T> Graph<T> 
    where T: PartialEq + Eq
{
    pub fn new<P>(mut links: Vec<Link<T, P>>) -> Self 
        where P: Ord
    {
        links.sort();

        let mut nodes: Vec<Mutex<Rc<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let to_node = if let Some(to_node) = nodes.iter().find(|node| node.lock().unwrap().data == to) {
                // Would Cause Cycle, Skip.
                let to_node = to_node.lock().unwrap();
                if to_node.contains_child(&from, Vec::new()) {
                    continue;
                }

                Rc::clone(&to_node)
            } else {
                let to_node = Rc::new(Node::new(to));
                nodes.push(Mutex::new(Rc::clone(&to_node)));
                to_node
            };

            to_node.ready.fetch_add(1, Ordering::AcqRel);

            if let Some(from_node) = nodes.iter().find(|node| node.lock().unwrap().data == from) {
                Rc::get_mut(&mut from_node.lock().unwrap()).unwrap().out_neighbours.push(to_node);
            } else {
                let from_node = Rc::new(Node::new(from));
                nodes.push(Mutex::new(Rc::clone(&from_node)));
            }
        }

        let nodes = nodes.into_iter().map(|node| {
            node.into_inner().unwrap()
        }).collect();

        Self {
            nodes
        }
    }
}