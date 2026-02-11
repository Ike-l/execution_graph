use std::{collections::HashSet, fmt::Debug, hash::Hash, sync::Arc};

use parking_lot::RwLock;
use tracing::{Level, event, span};

use crate::prelude::{Node, Link};

pub mod node;

pub struct Graph<T> {
    nodes: Vec<Arc<RwLock<Node<T>>>>,
}

impl<
    T: Debug
> Graph<T> {
    pub fn find_leaves(&self) -> Vec<Arc<RwLock<Node<T>>>> {
        self.nodes.iter().filter_map(|node| {
            if node.read().is_ready() {
                Some(Arc::clone(node))
            } else {
                None
            }
        }).collect()
    }

    pub fn nodes(&self) -> &Vec<Arc<RwLock<Node<T>>>> {
        &self.nodes
    }
}

impl<T> Graph<T> 
    where T: PartialEq + Eq + Debug + Hash
{
    /// assumes Links are sorted with priority at the end
    pub fn new(mut world: HashSet<T>, mut links: Vec<Link<T>>) -> Self {
        let span = span!(Level::DEBUG, "New Graph");
        let _enter = span.enter();

        let mut nodes: Vec<Arc<RwLock<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let span = span!(Level::TRACE, "Found Link", from =? from, to =? to);
            let _enter = span.enter();

            world.remove(&from);
            world.remove(&to);
            

            let to_node = if let Some(to_node) = nodes
                .iter()
                .find(|node| 
                    *node
                        .read()
                        .data() == to
                ) {
                event!(Level::TRACE, "Found 'to' Node");
                
                if to_node.read().contains_child(&from, Vec::new()) {
                    event!(Level::TRACE, "Found cycle");
                    
                    continue;
                }

                event!(Level::TRACE, "No cycle found");

                Arc::clone(&to_node)
            } else {
                event!(Level::TRACE, "No 'to' Node");
                
                let to_node = Arc::new(RwLock::new(Node::new(to)));
                nodes.push(Arc::clone(&to_node));
                to_node
            };

            to_node.write().make_unready();
            event!(Level::TRACE, "Made 'to' Node unready");

            if let Some(from_node) = nodes
                .iter()
                .find(|node| 
                    *node
                        .read()
                        .data() == from
                ) {
                event!(Level::TRACE, "Found 'from' Node");

                from_node.write().insert_out_neighbour(to_node);
            } else {
                event!(Level::TRACE, "No 'from' Node");

                let mut from_node = Node::new(from);
                from_node.insert_out_neighbour(to_node);

                let from_node = Arc::new(RwLock::new(from_node));
                nodes.push(Arc::clone(&from_node));
            }
        }

        let span = span!(Level::TRACE, "Checking Isolated Leaves");
        let _enter = span.enter();
        for isolated_leaf in world {
            event!(Level::TRACE, leaf =? isolated_leaf);
            
            let node = Arc::new(RwLock::new(Node::new(isolated_leaf)));
            nodes.push(node);
        }

        Self {
            nodes
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use parking_lot::RwLock;
    use proptest::{prelude::{Strategy, any, prop}, proptest};
    use tracing_subscriber::fmt;
    use std::{collections::HashSet, sync::{Arc, Once}};

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

    // test retained world makes leaves

    #[test]
    fn single_link() {
        let a = "A";
        let b = "B";

        let mut world = HashSet::new();
        world.insert(a);
        world.insert(b);

        let a_to_b = Link::new(a, b);
        
        let links = vec![a_to_b];

        let graph = Graph::new(world, links);
        
        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.read().data(), a);

        leaf.write().complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.read().data(), b);

        leaf.write().complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn double_link() {
        let a = "A";
        let b = "B";
        let c = "C";
        
        let a_to_b = Link::new(a, b);
        let b_to_c = Link::new(b, c);

        let mut world = HashSet::new();
        world.insert(a);
        world.insert(b);
        world.insert(c);

        let links = vec![a_to_b, b_to_c];

        let graph = Graph::new(world, links);

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.read().data(), a);

        leaf.write().complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.read().data(), b);

        leaf.write().complete();

        let mut leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 1);
        let leaf = leaves.remove(0);
        assert_eq!(*leaf.read().data(), c);

        leaf.write().complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn complex_scenario() {
        init_tracing();

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

        let mut world = HashSet::new();
        world.insert(a);
        world.insert(b);
        world.insert(c);
        world.insert(d);
        world.insert(e);
        world.insert(f);
        world.insert(g);
        world.insert(h);
        world.insert(i);
        world.insert(j);
        world.insert(k);
        
        let links = vec![
            Link::new(a, b), 
            Link::new(b, d), 
            Link::new(d, c), 
            Link::new(d, f), 
            Link::new(e, d), 
            Link::new(f, g), 
            Link::new(g, i), 
            Link::new(h, g), 
            Link::new(j, k), 
        ];

        let graph = Graph::new(world, links);

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            a.to_string(),
            e.to_string(),
            j.to_string(),
            h.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            k.to_string(),
            b.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            d.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            c.to_string(),
            f.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            g.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            i.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    #[test]
    fn complex_cycle() {
        let a = "A";
        let b = "B";
        let c = "C";
        let d = "D";
        let e = "E";
        let f = "F";
        let g = "G";

        let mut world = HashSet::new();
        world.insert(a);
        world.insert(b);
        world.insert(c);
        world.insert(d);
        world.insert(e);
        world.insert(f);
        world.insert(g);
        
        let links = vec![
            Link::new(a, b), 
            Link::new(b, c), 
            Link::new(c, d), 
            Link::new(e, f), 
            Link::new(c, e),

            Link::new(g, e), 

            Link::new(e, b), 
        ];

        let graph = Graph::new(world, links);

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            a.to_string(),
            c.to_string(),
            g.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            d.to_string(),
            e.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();

        let mut not_seen = HashSet::new();
        not_seen.extend([
            f.to_string(),
            b.to_string(),
        ]);

        for leaf in leaves {
            assert!(not_seen.contains(&leaf.read().data().to_string()));
            not_seen.remove(&leaf.read().data().to_string());  

            leaf.write().complete();
        }

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 0);
    }

    type I = u16;

    const SIZE: usize = 10000;

    fn chain_strategy() -> impl Strategy<Value = Vec<Link<I>>> {
        prop::collection::vec(
            (
                any::<I>(), 
                any::<I>(), 
            ).prop_map(|(from, to)| Link::new(from, to)),
            SIZE
        )
    }

    proptest! {
        #[test]
        fn build(input in chain_strategy()) {
            let world = get_world(&input);
            let graph = Graph::new(world.clone(), input);

            assert_eq!(world.len(), graph.nodes.len());
            assert!(!has_cycle(graph.nodes()))
        }
    }

    fn has_cycle(_nodes: &Vec<Arc<RwLock<Node<I>>>>) -> bool {
        false
    }

    fn get_world(input: &Vec<Link<I>>) -> HashSet<I> {
        input.iter().fold(HashSet::new(), |mut acc, cur| {
            acc.insert(cur.from);
            acc.insert(cur.to);
            acc
        })
    }
}