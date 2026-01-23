use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use bench_graph::prelude::*;

type T = u8;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Ordering {
    pub before: HashSet<T>,
    pub after: HashSet<T>,
    pub priority: f64
}

impl ExecutionOrdering for Ordering {
    type Item = T;
    fn consume(self) -> ( HashSet<Self::Item>, HashSet<Self::Item>, f64 ) {
        (self.before, self.after, self.priority)
    }

    fn subsume(&self, superset: &HashSet<Self::Item>) -> Self {
        Self {
            before: self.before.intersection(superset).cloned().collect(),
            after: self.after.intersection(superset).cloned().collect(),
            priority: self.priority
        } 
    }
}

pub fn criterion_abc(c: &mut Criterion) {
    // let mut group = c.
    let mut group = c.benchmark_group("Execution Graph Benchmark");
    group
        .sampling_mode(criterion::SamplingMode::Flat)
        .sample_size(10);

    let max: u32 = 4;
    let sizes = (0..max)
        .into_iter()
        .map(|i| 2_u32.pow(i))
        .collect::<Vec<u32>>();

    for size in sizes {
        let nodes = vec![];
        // group.bench_with_input(
        //     BenchmarkId::new("Size", size), 
        //     input, 
        //     |bencher, input| {
        //         bencher.
        //     }
        // )
        group.bench_function(
            BenchmarkId::new("Size", size), 
            move |bencher| {
                ExecutionGraph::new(nodes);
            }
        );
    }
}