# Architecture

## Data Flow

```
User Input                    Internal                          Output
─────────────────────────────────────────────────────────────────────────
Edge list / petgraph DiGraph
        │
        ▼
   CsrGraph<W>  ───────► Algorithm Selection ──┐
   (CSR layout,           (density check)       │
    cache-aligned)                              │
                                                ▼
                              ┌─────────────────────────────┐
                              │  density > 0.25 or n < 1024 │
                              │    YES → dijkstra()         │
                              │    NO  → dmmsy_core()       │
                              └─────────────────────────────┘
                                                │
                                                ▼
                                       ShortestPaths<W>
                                       { distances, predecessors }
```

## Module Dependency Graph

```
lib.rs (public API, re-exports, auto-selection)
  ├── graph.rs        CsrGraph<W> — CSR representation
  │     └── weight.rs   Weight trait (f32/f64/u32/u64)
  ├── dmmsy.rs        Core DMMSY algorithm
  │     ├── heap.rs     4-ary heap (cache-friendly)
  │     └── graph.rs    (uses CsrGraph)
  ├── dijkstra.rs     Baseline Dijkstra + ShortestPaths type
  │     ├── heap.rs     (shared 4-ary heap)
  │     └── graph.rs    (uses CsrGraph)
  ├── petgraph_compat.rs  [feature = "petgraph"]
  │     └── graph.rs    From<DiGraph> + DmmsyExt trait
  └── dimacs.rs       DIMACS format parser
```

## Memory Layout: CsrGraph

Compressed Sparse Row (CSR) stores edges in contiguous arrays for cache-friendly traversal:

```
offsets:  [0, 3, 5, 8, 10]     ← node i's edges start at offsets[i]
targets:  [1, 2, 4, 0, 3, ...]  ← target node IDs, packed sequentially
weights:  [1.0, 5.0, 2.0, ...]  ← weights, parallel to targets

Node 0: edges to targets[0..3] with weights weights[0..3]
Node 1: edges to targets[3..5] with weights weights[3..5]
```

Sequential memory access when iterating neighbors — critical for the hot edge-relaxation loop.

## 4-Ary Heap vs Binary Heap

Binary heap: children at `2i+1`, `2i+2` — span two cache lines for large heaps.
4-ary heap: children at `4i+1..4i+4` — fit in one cache line.

Result: ~2x fewer cache misses in priority queue operations that dominate SSSP runtime.

Both DMMSY and Dijkstra use the same 4-ary heap, so benchmarks compare algorithms, not data structures.

## Workspace Pre-Allocation

All scratch memory is allocated once in a `Workspace` struct before the algorithm runs. This avoids heap allocation in the hot loop and enables reuse across calls.

```rust
struct Workspace {
    frontier: Vec<u32>,
    short_range: Vec<u32>,
    long_range: Vec<u32>,
    dist_scratch: Vec<f64>,
    visited: Vec<bool>,
}
```

## Feature Flags

Optional dependencies behind feature gates. Default is `std` only:

- `petgraph` — Adds `From<DiGraph>` and `DmmsyExt` extension trait
- `serde` — Adds Serialize/Deserialize for `CsrGraph` and `ShortestPaths`
