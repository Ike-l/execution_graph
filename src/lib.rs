pub mod link;
pub mod graph;

pub mod prelude {
    pub use std::sync;

    pub use super::{
        graph::{
            Graph,
            node::{
                Node
            }
        },
        link::{
            Link
        }
    };
}