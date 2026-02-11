use std::collections::HashSet;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use execution_graph::prelude::*;
use rand::random_range;

type T = u16;
const SAMPLE_SIZE: usize = 10;
const RESOLUTION: usize = 10;

const WORLD_SIZES: [usize; RESOLUTION] = world_size_builder();
const SPARSE_SIZE: usize = 4; // halves connections into 2 (links have 2 nodes) - 1 in every 2 nodes has a link
const DENSE_SIZE: usize = 1; // doubles connections into 2 (links have 2 nodes) - 1 in every 2 nodes has a link

const fn world_size_builder() -> [usize; RESOLUTION] {
    [
        1, 4, 16, 64, 256, 1024, 4096, 16384, 65536, 262144
    ]
}

pub fn build_sparse_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("Execution Graph Benchmark");
    group
        .sampling_mode(criterion::SamplingMode::Auto)
        .sample_size(SAMPLE_SIZE);


    for world_size in WORLD_SIZES {
        let world: HashSet<T> = generate_world(world_size);
        let links: Vec<Link<T>> = generate_chain(&world, world_size / SPARSE_SIZE);

        group.bench_with_input(
            BenchmarkId::new(format!("Build Sparse"), format!("World Size: {world_size}")), 
            &(world, links),
            |bencher, (world, links)| {
                bencher.iter(|| Graph::new(world.clone(), links.clone()));
            }
        );
    }
}

pub fn generate_world(world_size: usize) -> HashSet<T> {
    rand::random_iter().take(world_size).collect()
}

pub fn generate_chain(world: &HashSet<T>, chain_size: usize) -> Vec<Link<T>> {
    let world = world.iter().collect::<Vec<_>>();
    let len = world.len();
    
    (0..chain_size).map(|_| {
        let from = world[random_range(0..len)].clone();
        let to = world[random_range(0..len)].clone();

        Link::new(from, to)
    }).collect()
}

pub fn build_dense_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("Execution Graph Benchmark");
    group
        .sampling_mode(criterion::SamplingMode::Auto)
        .sample_size(SAMPLE_SIZE);


    for world_size in WORLD_SIZES {
        let world: HashSet<T> = generate_world(world_size);
        let links: Vec<Link<T>> = generate_chain(&world, world_size * DENSE_SIZE);

        group.bench_with_input(
            BenchmarkId::new(format!("Build Dense"), format!("World Size: {world_size}")), 
            &(world, links),
            |bencher, (world, links)| {
                bencher.iter(|| Graph::new(world.clone(), links.clone()));
            }
        );
    }
}


criterion_group!(benches, build_sparse_graph, build_dense_graph);
criterion_main!(benches);