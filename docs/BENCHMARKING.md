# Benchmarking Guide

## Running Benchmarks

```sh
# All benchmarks
cargo bench

# Specific benchmark group
cargo bench -- sparse-random
cargo bench -- dense-random

# With HTML reports (open target/criterion/report/index.html)
cargo bench
open target/criterion/report/index.html
```

## Benchmark Configurations

### Sparse Random Graphs

| Config | Nodes | Edges | Density | Edges/Node |
|---|---|---|---|---|
| sparse-10K | 10,000 | 40,000 | 0.0004 | 4 |
| sparse-100K | 100,000 | 400,000 | 0.00004 | 4 |

### Dense Random Graphs

| Config | Nodes | Edges | Density | Edges/Node |
|---|---|---|---|---|
| dense-500 | 500 | 62,500 | 0.25 | 125 |
| dense-1K | 1,000 | 250,000 | 0.25 | 250 |

### DIMACS Road Networks (Requires Data Files)

| Network | Nodes | Edges | File |
|---|---|---|---|
| New York | 264,346 | 730,100 | USA-road-d.NY.gr |
| Florida | 1,070,376 | 2,712,798 | USA-road-d.FLA.gr |

Download from: http://www.dis.uniroma1.it/challenge9/download.shtml
Place in `data/` directory.

## What's Being Compared

1. **shortest_paths()** — Auto-selecting algorithm (DMMSY or Dijkstra)
2. **dijkstra()** — Our Dijkstra with 4-ary heap on CSR graph

Both use the same data structures (CsrGraph + FourAryHeap), so the benchmark isolates the algorithm difference.

## Profiling

```sh
# Flamegraph (requires cargo-flamegraph)
cargo flamegraph --bench comparison -- --bench sparse-random

# perf stat
cargo bench -- sparse-random --profile-time 10
```

## Reproducing Results

All benchmarks use `Xoshiro256StarStar` with seed `42` for deterministic graph generation. Results should be reproducible across runs on the same hardware.

## Expected Results

- **Sparse graphs (>100K nodes):** DMMSY should outperform Dijkstra
- **Dense/small graphs:** Dijkstra should win (lower constant factors)
- **Road networks:** DMMSY advantage depends on graph structure
- **Both algorithms** should outperform petgraph's Dijkstra due to CSR layout + 4-ary heap
