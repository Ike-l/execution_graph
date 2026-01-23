mod execution_graph;
mod node;
mod ordering;

pub mod prelude {
    pub use super::{
        node::Node,
        ordering::{
            ExecutionOrdering, 
        },
        execution_graph::ExecutionGraph,
    };

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub struct SystemId(pub String);
}
