# dmmsy

**Usable DMMSY shortest paths for Rust.**

The first *usable* Rust crate implementing the DMMSY algorithm from [STOC 2025](https://arxiv.org/abs/2504.17033) that broke Dijkstra's 66-year O(m + n log n) barrier.

## Why this crate?

Other implementations exist but are benchmark-only binaries. This crate is designed for real use:

- **Drop-in petgraph integration** — `From<DiGraph>` + `DmmsyExt` trait
- **Generic weight types** — f32, f64, u32, u64
- **Auto-algorithm selection** — DMMSY on sparse graphs, Dijkstra fallback on dense
- **Published on crates.io** — `cargo add dmmsy`

## Quick Start

```rust
use dmmsy::{CsrGraph, shortest_paths};

let graph = CsrGraph::from_edges(4, &[
    (0, 1, 1.0), (1, 2, 2.0), (0, 2, 5.0), (2, 3, 1.0),
]);
let result = shortest_paths(&graph, 0);
assert_eq!(result.distances[3], 4.0); // 0 → 1 → 2 → 3
```

## With petgraph

```toml
[dependencies]
dmmsy = { version = "0.1", features = ["petgraph"] }
```

```rust
use petgraph::graph::DiGraph;
use dmmsy::petgraph_compat::DmmsyExt;

let mut g = DiGraph::new();
let a = g.add_node("A");
let b = g.add_node("B");
let c = g.add_node("C");
g.add_edge(a, b, 1.0_f64);
g.add_edge(b, c, 2.0);

let paths = g.dmmsy_shortest_paths(a);
assert_eq!(paths.distance(c.index() as u32), 3.0);
```

## Demo

![demo](demo.gif)

Run it locally:
```sh
./demo.sh
```

## When to use DMMSY vs Dijkstra

| Graph type | Best algorithm | Why |
|---|---|---|
| Large sparse (>100K nodes, ~4 edges/node) | DMMSY | Asymptotic advantage |
| Small (<1K nodes) | Dijkstra | Lower constant factors |
| Dense (>25% edge density) | Dijkstra | DMMSY overhead not worth it |

`shortest_paths()` auto-selects based on graph characteristics.

## Features

| Feature | Default | Description |
|---|---|---|
| `std` | Yes | Standard library support |
| `petgraph` | No | petgraph `From<DiGraph>` + `DmmsyExt` |
| `serde` | No | Serialize/deserialize graphs and results |

## Benchmarks

| Benchmark | DMMSY (`shortest_paths_f64`) | Dijkstra |
|-----------|------------------------------|----------|
| Sparse 10K nodes, 4 edges/node | 1.61 ms | 1.62 ms |
| Sparse 100K nodes, 4 edges/node | 36.99 ms | 33.47 ms |
| Dense 500 nodes, 25% | 105.86 us | 105.39 us |
| Dense 1K nodes, 25% | 371.05 us | 369.45 us |

> DMMSY's theoretical advantage shows on very large sparse graphs (>1M nodes). On smaller graphs, Dijkstra's simpler constant factors dominate. The `shortest_paths_f64()` function automatically selects the best algorithm.

To run benchmarks yourself:

```sh
cargo bench
```

Benchmarks compare DMMSY vs Dijkstra (4-ary heap) on:
- Sparse random graphs (10K, 100K nodes)
- Dense random graphs (500, 1K nodes)
- DIMACS road networks (NY, FLA) — requires data files

## Project Structure

```
src/
├── lib.rs              Public API, auto-selection
├── graph.rs            CsrGraph — CSR representation
├── weight.rs           Weight trait (f32/f64/u32/u64)
├── dmmsy.rs            Core DMMSY algorithm
├── dijkstra.rs         Dijkstra baseline + ShortestPaths
├── heap.rs             4-ary heap (cache-friendly)
├── petgraph_compat.rs  petgraph integration
└── dimacs.rs           DIMACS format parser
```

## Credits

The core DMMSY algorithm (`ssp_duan` + `bmsp_rec`) is adapted from
[danalec/DMMSY-SSSP-rs](https://github.com/danalec/DMMSY-SSSP-rs).
This crate wraps that proven implementation with a library API,
petgraph integration, generic weights, and documentation.

## References

- **Paper:** [Breaking the Sorting Barrier for Directed Single-Source Shortest Paths](https://arxiv.org/abs/2504.17033) — Duan, Mao, Mao, Shu, Yin (STOC 2025)
- **Rust implementation:** [danalec/DMMSY-SSSP-rs](https://github.com/danalec/DMMSY-SSSP-rs)
- **C99 implementation:** [danalec/DMMSY-SSSP](https://github.com/danalec/DMMSY-SSSP)


