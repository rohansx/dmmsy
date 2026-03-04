use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use dmmsy::{CsrGraph, dijkstra, shortest_paths};
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

/// Generate a random sparse directed graph.
fn random_sparse_graph(num_nodes: usize, num_edges: usize, seed: u64) -> CsrGraph<f64> {
    let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
    let mut edges = Vec::with_capacity(num_edges);
    for _ in 0..num_edges {
        let src = rng.gen_range(0..num_nodes as u32);
        let tgt = rng.gen_range(0..num_nodes as u32);
        let weight: f64 = rng.gen_range(1.0..100.0);
        edges.push((src, tgt, weight));
    }
    CsrGraph::from_edges(num_nodes, &edges)
}

fn bench_sparse_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse-random");
    group.sample_size(10);

    for &n in &[10_000, 100_000] {
        let m = n * 4; // ~4 edges per node (sparse)
        let g = random_sparse_graph(n, m, 42);

        group.bench_with_input(
            BenchmarkId::new("shortest_paths", n),
            &g,
            |b, g| b.iter(|| shortest_paths(g, 0)),
        );
        group.bench_with_input(
            BenchmarkId::new("dijkstra", n),
            &g,
            |b, g| b.iter(|| dijkstra(g, 0)),
        );
    }
    group.finish();
}

fn bench_dense_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("dense-random");
    group.sample_size(10);

    for &n in &[500, 1_000] {
        let m = n * n / 4; // 25% density
        let g = random_sparse_graph(n, m, 42);

        group.bench_with_input(
            BenchmarkId::new("shortest_paths", n),
            &g,
            |b, g| b.iter(|| shortest_paths(g, 0)),
        );
        group.bench_with_input(
            BenchmarkId::new("dijkstra", n),
            &g,
            |b, g| b.iter(|| dijkstra(g, 0)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_sparse_random, bench_dense_random);
criterion_main!(benches);
