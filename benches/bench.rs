use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use bench_graph::prelude::*;
use rand::random;

type T = u128;

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

pub fn bench_graph_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("Execution Graph Benchmark");
    group
        .sampling_mode(criterion::SamplingMode::Flat)
        .sample_size(10);

    let max: u32 = 11;
    let test_sizes = (0..max)
        .into_iter()
        .map(|i| 2_usize.pow(i))
        .collect::<Vec<_>>();

    let set_max: u32 = 3;
    let test_set_sizes = (0..set_max)
        .into_iter()
        .map(|i| 2_usize.pow(i))
        .collect::<Vec<_>>();


    for (size, set_size) in create_permutations(&test_sizes, &test_set_sizes) {
        let nodes = generate_nodes(size, set_size);
        let nodes = nodes.iter().map(|(s, o)| (s.clone(), o)).collect::<Vec<_>>();
        group.bench_with_input(
            BenchmarkId::new(format!("Size: {size}"), format!("Set Size: {set_size}")), 
            &nodes,
            |bencher, input| {
                bencher.iter(|| ExecutionGraph::new(input));
            }
        );
    }
}

fn create_permutations(test_sizes: &[usize], test_set_sizes: &[usize]) -> Vec<(usize, usize)> {
    test_sizes
        .iter()
        .map(
            |size_a| {
                test_set_sizes.iter().map(|size_b| {
                    (*size_a, *size_b)
                })
        })
        .flatten()
        .collect()
}

fn generate_nodes(size: usize, set_size: usize) -> Vec<(T, Ordering)> {
    let mut result = Vec::with_capacity(size);
    for _ in 0..size {
        let t = generate_random_t();
        let ordering = Ordering {
            before: generate_random_set(set_size),
            after: generate_random_set(set_size),
            priority: generate_random_priority()
        };

        result.push((t, ordering));
    }
    
    result
}

fn generate_random_priority() -> f64 {
    // random()
    0.0
}

fn generate_random_set(set_size: usize) -> HashSet<T> {
    rand::random_iter().take(set_size).collect()
}

fn generate_random_t() -> T {
    random()
}

criterion_group!(benches, bench_graph_create);
criterion_main!(benches);