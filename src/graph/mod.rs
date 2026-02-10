use std::{fmt::Debug, rc::Rc, sync::Mutex};

use tracing::{Level, event, span};

use crate::{graph::node::Node, link::Link};

pub mod node;

pub struct Graph<T> {
    nodes: Vec<Rc<Mutex<Node<T>>>>
}

impl<
    T: Debug
> Graph<T> {
    pub fn find_leaves(&self) -> Vec<Rc<Mutex<Node<T>>>> {
        self.nodes.iter().filter_map(|node| {
            if node.lock().unwrap().is_ready() {
                Some(Rc::clone(node))
            } else {
                None
            }
        }).collect()
    }

    pub fn nodes(&self) -> &Vec<Rc<Mutex<Node<T>>>> {
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

        let mut nodes: Vec<Rc<Mutex<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let span = span!(Level::TRACE, "Found Link", from =? from, to =? to);
            let _enter = span.enter();

            let to_node = if let Some(to_node) = nodes.iter().find(|node| *node.lock().unwrap().data() == to) {
                event!(Level::TRACE, "Found 'to' Node");
                
                if to_node.lock().unwrap().contains_child(&from, Vec::new()) {
                    event!(Level::TRACE, "Found cycle");
                    
                    continue;
                }

                event!(Level::TRACE, "No cycle found");

                Rc::clone(&to_node)
            } else {
                event!(Level::TRACE, "No 'to' Node");
                
                let to_node = Rc::new(Mutex::new(Node::new(to)));
                nodes.push(Rc::clone(&to_node));
                to_node
            };

            to_node.lock().unwrap().make_unready();
            event!(Level::TRACE, "Made 'to' Node unready");

            if let Some(from_node) = nodes.iter().find(|node| *node.lock().unwrap().data() == from) {
                event!(Level::TRACE, "Found 'from' Node");

                from_node.lock().unwrap().insert_out_neighbour(to_node);
            } else {
                event!(Level::TRACE, "No 'from' Node");

                let mut from_node = Node::new(from);
                from_node.insert_out_neighbour(to_node);

                let from_node = Rc::new(Mutex::new(from_node));
                nodes.push(Rc::clone(&from_node));
            }
        }

        Self {
            nodes
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{graph::Graph, link::Link};

    use proptest::{prelude::{Strategy, any, prop}, proptest};
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
        assert_eq!(*leaf.lock().unwrap().data(), a);

        leaf.lock().unwrap().complete();

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.lock().unwrap().data(), b);

        leaf.lock().unwrap().complete();

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
        assert_eq!(*leaf.lock().unwrap().data(), a);

        leaf.lock().unwrap().complete();

        let mut leaves = graph.find_leaves();
        // event!(Level::DEBUG, leaves =? leaves, "Leaves");

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.lock().unwrap().data(), b);

        leaf.lock().unwrap().complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.lock().unwrap().data(), c);

        leaf.lock().unwrap().complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn complex_scenario() {
        init_tracing();

        let span = span!(Level::DEBUG, "Complex Scenario");
        let _enter = span.enter();

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
        span.record("Links", format!("{links:?}"));

        let graph = Graph::new(links);
        event!(Level::DEBUG, nodes =? graph.nodes());

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            a.to_string(),
            e.to_string(),
            j.to_string(),
            h.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            k.to_string(),
            b.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            d.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            c.to_string(),
            f.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            g.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            i.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn complex_cycle() {
        init_tracing();

        let span = span!(Level::DEBUG, "Complex Cycle");
        let _enter = span.enter();

        let a = "A";
        let b = "B";
        let c = "C";
        let d = "D";
        let e = "E";
        let f = "F";
        let g = "G";
        
        let priority = 0;
        
        let links = vec![
            Link::new(a, b, priority), 
            Link::new(b, c, priority), 
            Link::new(c, d, priority), 
            Link::new(c, e, priority + 1), 
            Link::new(e, b, priority + 2), 
            Link::new(e, f, priority), 
            Link::new(g, e, priority), 
        ];
        span.record("Links", format!("{links:?}"));

        let graph = Graph::new(links);
        event!(Level::DEBUG, nodes =? graph.nodes());

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            a.to_string(),
            c.to_string(),
            g.to_string(),
        ]);

        for leaf in leaves {
            let span = span!(Level::DEBUG, "Leaf", data =? leaf.lock().unwrap().data());
            let _enter = span.enter();
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            d.to_string(),
            e.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        event!(Level::DEBUG, leaves =? leaves, "Leaves");

        let mut not_seen = HashSet::new();
        not_seen.extend([
            f.to_string(),
            b.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.lock().unwrap().data().to_string()));
            not_seen.remove(&leaf.lock().unwrap().data().to_string());  

            leaf.lock().unwrap().complete();
        }

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    type I = u16;
    type P = u64;

    const SIZE: usize = 10000;

    fn chain_strategy() -> impl Strategy<Value = Vec<Link<I, P>>> {
        prop::collection::vec(
            (
                any::<I>(), 
                any::<I>(), 
                any::<P>()
            ).prop_map(|(from, to, priority)| Link::new(from, to, priority)),
            SIZE
        )
    }

    proptest! {
        #[test]
        fn build(input in chain_strategy()) {
            let nodes = get_unique_nodes(&input);
            let graph = Graph::new(input);

            // make sure every node in input is used
            assert_eq!(nodes.len(), graph.nodes.len());
            
            // check for cycles
            // make sure they are trees
        }
    }

    fn get_unique_nodes(input: &Vec<Link<I, P>>) -> HashSet<I> {
        input.iter().fold(HashSet::new(), |mut acc, cur| {
            acc.insert(cur.from);
            acc.insert(cur.to);
            acc
        })
    }
}