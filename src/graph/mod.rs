use std::{fmt::Debug, rc::Rc, sync::Mutex};

use tracing::{Level, event, span};

use crate::{graph::node::Node, link::Link};

pub mod node;

pub struct Graph<T> {
    nodes: Vec<Rc<Node<T>>>
}

impl<
    T: Debug
> Graph<T> {
    pub fn find_leaves(&self) -> Vec<Rc<Node<T>>> {
        self.nodes.iter().filter_map(|node| {
            if node.is_ready() {
                Some(Rc::clone(node))
            } else {
                None
            }
        }).collect()
    }

    pub fn nodes(&self) -> &Vec<Rc<Node<T>>> {
        &self.nodes
    }
}

impl<T> Graph<T> 
    where T: PartialEq + Eq + Debug
{
    pub fn new<P>(mut links: Vec<Link<T, P>>) -> Self 
        where P: Ord
    {
        let span = span!(Level::DEBUG, "New Graph");
        let _enter = span.enter();

        links.sort();
        event!(Level::TRACE, "Sorted links");

        let mut nodes: Vec<Mutex<Rc<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let span = span!(Level::TRACE, "Found Link", from =? from, to =? to);
            let _enter = span.enter();

            let to_node = if let Some(to_node) = nodes.iter().find(|node| *node.lock().unwrap().data() == to) {
                event!(Level::TRACE, "Found to Node");
                
                let to_node = to_node.lock().unwrap();
                if to_node.contains_child(&from, Vec::new()) {
                    event!(Level::TRACE, "Found cycle");
                    
                    continue;
                }

                event!(Level::TRACE, "No cycle found");

                Rc::clone(&to_node)
            } else {
                event!(Level::TRACE, "No to Node");
                
                let to_node = Rc::new(Node::new(to));
                nodes.push(Mutex::new(Rc::clone(&to_node)));
                to_node
            };

            to_node.make_unready();
            event!(Level::TRACE, "Made to Node unready");

            if let Some(from_node) = nodes.iter().find(|node| *node.lock().unwrap().data() == from) {
                event!(Level::TRACE, "Found from Node");

                Rc::get_mut(&mut from_node.lock().unwrap()).unwrap().insert_out_neighbour(to_node);
            } else {
                event!(Level::TRACE, "No from Node");

                let mut from_node = Node::new(from);
                from_node.insert_out_neighbour(to_node);

                let from_node = Rc::new(from_node);
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

#[cfg(test)]
mod tests {
    use crate::{graph::Graph, link::Link};

    use tracing::{Level, event, span};
    use tracing_subscriber::fmt;
    use std::{collections::HashSet, sync::Once};

    static INIT: Once = Once::new();

    fn init_tracing() {
        INIT.call_once(|| {
            fmt()
                .with_max_level(tracing::Level::TRACE)
                .with_target(false)
                .with_test_writer()           
                .init();
        });
    }

    #[test]
    fn single_link() {
        // init_tracing();

        // let span = span!(Level::DEBUG, "Single Link");
        // let _enter = span.enter();

        let a = "A";
        let b = "B";

        let priority = 0;

        let a_to_b = Link::new(a, b, priority);
        
        let links = vec![a_to_b];
        // span.record("Links", format!("{links:?}"));

        let graph = Graph::new(links);
        // event!(Level::DEBUG, nodes =? graph.nodes());

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), a);

        leaf.complete();
        // event!(Level::DEBUG, "Completed");

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), b);

        leaf.complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn double_link() {
        // init_tracing();

        // let span = span!(Level::DEBUG, "Single Link");
        // let _enter = span.enter();

        let a = "A";
        let b = "B";
        
        let priority = 0;
        let a_to_b = Link::new(a, b, priority);

        let c = "C";

        let b_to_c = Link::new(b, c, priority);
        
        let links = vec![a_to_b, b_to_c];
        // span.record("Links", format!("{links:?}"));

        let graph = Graph::new(links);
        // event!(Level::DEBUG, nodes =? graph.nodes());

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), a);

        leaf.complete();
        // event!(Level::DEBUG, "Completed");

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), b);

        leaf.complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), c);

        leaf.complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn complex_scenario() {
        // init_tracing();

        // let span = span!(Level::DEBUG, "Complex Scenario");
        // let _enter = span.enter();

        let a = "A";
        let b = "B";
        let c = "C";
        let d = "D";
        let e = "E";
        let f = "F";
        let g = "G";
        let h = "H";
        let i = "I";
        let j = "J";
        let k = "K";
        
        let priority = 0;
        
        let links = vec![
            Link::new(a, b, priority), 
            Link::new(b, d, priority), 
            Link::new(d, c, priority), 
            Link::new(d, f, priority), 
            Link::new(e, d, priority), 
            Link::new(f, g, priority), 
            Link::new(g, i, priority), 
            Link::new(h, g, priority), 
            Link::new(j, k, priority), 
        ];
        // span.record("Links", format!("{links:?}"));

        let graph = Graph::new(links);
        // event!(Level::DEBUG, nodes =? graph.nodes());

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 3);
        let mut not_seen = HashSet::from_iter(vec![a, e, j]);
        for leaf in leaves {
            assert!(not_seen.contains(leaf.data()));
            not_seen.remove(leaf.data());  

            leaf.complete();
        }

        // event!(Level::DEBUG, "Completed");

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), b);

        leaf.complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.data(), c);

        leaf.complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }
}