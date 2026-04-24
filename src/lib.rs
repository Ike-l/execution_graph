pub mod link;
pub mod graph;

pub mod prelude {
    pub use super::{
        graph::{
            Graph,
            flow::{
                Flow
            },
            node::{
                Node,
                status::{
                    Status
                }
            }
        },
        link::{
            Link,
        }
    };
}