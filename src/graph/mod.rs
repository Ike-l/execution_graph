use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, sync::Arc};

use parking_lot::RwLock;
use tracing::{Level, event, span};

use crate::prelude::{Node, Link};

pub mod node;
pub mod flow;

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
    where T: Debug + PartialEq + Eq + Hash + Clone
{
    /// Automatically chooses the best implementation based on input size & benchmarking
    /// 
    /// assumes Links are sorted with priority at the end
    /// 
    /// assumes links is a subset of world
    pub fn new(world: HashSet<T>, links: Vec<Link<T>>) -> Self {
        if links.len() < 64 {
            Self::new_3(world, links)
        } else {
            Self::new_2(world, links)
        }
    }

    /// Uses HashSet implementation
    /// 
    /// assumes Links are sorted with priority at the end
    /// 
    /// assumes links is a subset of world
    pub fn new_2(mut world: HashSet<T>, mut links: Vec<Link<T>>) -> Self {
        let span = span!(Level::INFO, "New Graph");
        let _enter = span.enter();

        let mut nodes: HashMap<T, Arc<RwLock<Node<T>>>> = HashMap::with_capacity(links.len());
        // let mut nodes: Vec<Arc<RwLock<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let span = span!(Level::DEBUG, "Found Link", from =? from, to =? to);
            let _enter = span.enter();

            event!(Level::DEBUG, "Processing");

            if to == from {
                event!(Level::WARN, "Link is intangible");
                continue;
            }

            world.remove(&to);
            
            let to_node = if let Some(to_node) = nodes.get(&to) {
            // let to_node = if let Some(to_node) = nodes
            // .iter()
            // .find(|node| 
            //     *node
            //     .read()
            //     .data() == to
            // ) {
                event!(Level::TRACE, "Found 'to' Node");
                
                if to_node.read().contains_child(&from, &mut HashSet::new()) {
                    event!(Level::WARN, "Found cycle");
                    
                    continue;
                }
                
                event!(Level::TRACE, "No cycle found");
                
                Arc::clone(&to_node)
            } else {
                event!(Level::TRACE, "No 'to' Node");
                
                let to_node = Arc::new(RwLock::new(Node::new(to.clone())));
                // let to_node = Arc::new(RwLock::new(Node::new(to)));
                nodes.insert(to, Arc::clone(&to_node));
                // nodes.push(Arc::clone(&to_node));
                to_node
            };

            world.remove(&from);

            to_node.write().make_unready();
            event!(Level::TRACE, "Made 'to' Node unready");

            if let Some(from_node) = nodes.get(&from) {
            // if let Some(from_node) = nodes
            //     .iter()
            //     .find(|node| 
            //         *node
            //             .read()
            //             .data() == from
            //     ) {
                event!(Level::TRACE, "Found 'from' Node");

                from_node.write().insert_out_neighbour(to_node);
            } else {
                event!(Level::TRACE, "No 'from' Node");

                let mut from_node = Node::new(from.clone());
                // let mut from_node = Node::new(from);
                from_node.insert_out_neighbour(to_node);

                let from_node = Arc::new(RwLock::new(from_node));
                nodes.insert(from, Arc::clone(&from_node));
                // nodes.push(Arc::clone(&from_node));
            }
        }

        let span = span!(Level::INFO, "Checking Isolated Leaves");
        let _enter = span.enter();
        for isolated_leaf in world {
            event!(Level::DEBUG, leaf =? isolated_leaf);
            
            let node = Arc::new(RwLock::new(Node::new(isolated_leaf.clone())));
            // let node = Arc::new(RwLock::new(Node::new(isolated_leaf)));
            nodes.insert(isolated_leaf, node);
            // nodes.push(node);
        }

        let nodes = nodes.into_iter().map(|(_, node)| node).collect();

        Self {
            nodes
        }
    }
}

impl<T> Graph<T> 
    where T: Debug + PartialEq + Eq + Hash 
{
    /// Uses Vec implementation
    /// 
    /// assumes Links are sorted with priority at the end
    /// 
    /// assumes links is a subset of world
    pub fn new_3(mut world: HashSet<T>, mut links: Vec<Link<T>>) -> Self {
        let span = span!(Level::INFO, "New Graph");
        let _enter = span.enter();

        let mut nodes: Vec<Arc<RwLock<Node<T>>>> = Vec::with_capacity(links.len());
        while let Some(Link { from, to, ..}) = links.pop() {
            let span = span!(Level::DEBUG, "Found Link", from =? from, to =? to);
            let _enter = span.enter();

            event!(Level::DEBUG, "Processing");

            if to == from {
                event!(Level::WARN, "Link is intangible");
                continue;
            }

            world.remove(&to);
            
            let to_node = if let Some(to_node) = nodes
            .iter()
            .find(|node| 
                *node
                .read()
                .data() == to
            ) {
                event!(Level::TRACE, "Found 'to' Node");
                
                if to_node.read().contains_child_2(&from, Vec::new()) {
                    event!(Level::WARN, "Found cycle");
                    
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

            world.remove(&from);

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

        let span = span!(Level::INFO, "Checking Isolated Leaves");
        let _enter = span.enter();
        for isolated_leaf in world {
            event!(Level::DEBUG, leaf =? isolated_leaf);
            
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

    use proptest::prelude::*;
    use tracing::{Level, event, span};
    use tracing_subscriber::fmt;

    use std::{collections::HashSet, sync::Once};

    static INIT: Once = Once::new();

    fn init_tracing() {
        INIT.call_once(|| {
            fmt()
                .with_ansi(false)
                .without_time()
                .with_target(false)

                .with_max_level(tracing::Level::TRACE)
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

        let a_to_b = Link::cheap(a, b);
        
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
        
        let a_to_b = Link::cheap(a, b);
        let b_to_c = Link::cheap(b, c);

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
            Link::cheap(a, b), 
            Link::cheap(b, d), 
            Link::cheap(d, c), 
            Link::cheap(d, f), 
            Link::cheap(e, d), 
            Link::cheap(f, g), 
            Link::cheap(g, i), 
            Link::cheap(h, g), 
            Link::cheap(j, k), 
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
            Link::cheap(a, b), 
            Link::cheap(b, c), 
            Link::cheap(c, d), 
            Link::cheap(e, f), 
            Link::cheap(c, e),

            Link::cheap(g, e), 

            Link::cheap(e, b), 
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

    #[test]
    fn simple_cycle() {
        init_tracing();
        let a = "A";
        let b = "B";

        let links = vec![
            Link::cheap(a, b),
            Link::cheap(b, a)
        ];

        let mut world = HashSet::new();
        world.insert(a);
        world.insert(b);

        let graph = Graph::new(world, links); 

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 1);

        let leaf = leaves.first().unwrap();
        assert_eq!(leaf.read().data().to_string(), b.to_string());

        leaf.write().complete();

        let leaves = graph.find_leaves();
        assert_eq!(leaves.len(), 1);

        let leaf = leaves.first().unwrap();
        assert_eq!(leaf.read().data().to_string(), a.to_string());

        leaf.write().complete();

        let leaves = graph.find_leaves();

        assert_eq!(leaves.len(), 0);
    }

    type I = u8;

    const CHAIN_SIZE: usize = 2000;
    const WORLD_SIZE: usize = 10000;

    fn chain_strategy(world: HashSet<I>) -> impl Strategy<Value = Vec<Link<I>>> {
        let world = world.into_iter().collect::<Vec<_>>();
        prop::collection::vec(
            (
                prop::sample::select(world.clone()), 
                prop::sample::select(world), 
            ).prop_map(|(from, to)| Link::cheap(from, to)),
            1..=CHAIN_SIZE
        )
    }

    fn world_strategy() -> impl Strategy<Value = HashSet<I>> {
        prop::collection::hash_set(
            any::<I>(), 
            1..=WORLD_SIZE
        )
    }

    fn input_strategy() -> impl Strategy<Value = (HashSet<I>, Vec<Link<I>>)> {
        world_strategy().prop_flat_map(|world| {
            let chain = chain_strategy(world.clone());
            (prop::strategy::Just(world), chain)
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            max_shrink_iters: 100_000,
            .. ProptestConfig::default()
        })]

        #[test]
        fn build((world, chain) in input_strategy()) {
            // init_tracing();

            let graph = Graph::new_2(world.clone(), chain);

            assert_eq!(world.len(), graph.nodes.len());
            assert!(complete_forest(world, &graph))
            // assert!(!has_cycle(graph.nodes()))
        }
    }

    // All nodes are visited
    // A node is only visited Once
    fn complete_forest(
        mut world: HashSet<I>, 
        graph: &Graph<I>, 
    ) -> bool {
        let span = span!(Level::INFO, "Checking Graph");
        let _enter = span.enter();

        let mut count = 0;
        loop {
            let mut leaves = graph.find_leaves();
            if leaves.is_empty() {
                event!(Level::INFO, "Completed Leaves");
                break;
            }

            for _ in 0..2 {
                let Some(leaf) = leaves.pop() else { break; };
                
                let mut leaf = leaf.write();
                
                let data = leaf.data();
                let out_degree = leaf.out_degree();
                
                let in_degree = leaf.in_degree();
                assert_eq!(in_degree, 0);
    
                let span = span!(Level::DEBUG, "Leaf", data = data, out_degree = out_degree);
                let _enter = span.enter();
    
                event!(Level::INFO, count = count);
    
                assert!(world.contains(&data));
                world.remove(&data);
    
                leaf.complete();
                count += 1;
            }
        };

        world.is_empty()
    }
}