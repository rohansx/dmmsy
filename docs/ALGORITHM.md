# DMMSY Algorithm

## Paper Reference

**"Breaking the Sorting Barrier for Directed Single-Source Shortest Paths"**
Ran Duan, Jiayi Mao, Xiao Mao, Xinkai Shu, Longhui Yin
STOC 2025 | [arXiv:2504.17033](https://arxiv.org/abs/2504.17033)

## The Sorting Barrier

For 66 years, Dijkstra's algorithm was the best at O(m + n log n). The bottleneck is fundamental: Dijkstra implicitly sorts all vertices by distance from the source. Since comparison-based sorting requires Ω(n log n), this seemed unbeatable.

## The DMMSY Breakthrough

**Complexity:** O(m · log^(2/3) n) deterministic, comparison-addition model.

**Core insight:** SSSP does NOT require sorting — only distance computation. You don't need a total ordering of vertices by distance; a partial ordering suffices.

## Algorithm Overview

### 1. Pivot Selection

Instead of maintaining every candidate vertex in a global priority queue, identify "pivots" — vertices that root substantial shortest-path subtrees.

- Run limited Bellman-Ford-like relaxation steps from source
- Vertices with large subtrees become pivots
- This shrinks the active frontier by a log^Ω(1)(n) factor

### 2. Frontier Decomposition

Split remaining vertices into buckets:
- **Short-range:** Close to a pivot — solved cheaply via local search
- **Long-range:** Far from all pivots — handled recursively

### 3. Recursive BMSSP

BMSSP (Bounded Multi-Source Shortest Path): compute distances from a source set S, up to distance bound B.

- Problem partitioned into ~(log n)/t hierarchical levels
- Each level solved recursively
- Novel "block-based linked lists with batch prepend" data structure enables efficient bulk insertion

### 4. Partial-Order Priority Queue

Within each processing window:
- Use a partial-order queue (cheaper than full comparison heap)
- Process nodes in approximate order
- Only enough ordering for correctness — stop over-sorting

## Practical Implications

| Graph Characteristic | DMMSY wins? | Notes |
|---|---|---|
| Large sparse (n > 100K, m ≈ 4n) | Yes | Asymptotic advantage kicks in |
| Road networks (sparse, millions of nodes) | Likely | Real-world validation needed |
| Small graphs (n < 1K) | No | Dijkstra's lower constant factors win |
| Dense graphs (density > 25%) | No | DMMSY's overhead exceeds savings |

## Porting Notes from C99

### Key C99 Functions to Rust Mapping

| C99 Function | Rust Module | Notes |
|---|---|---|
| `ssp_duan()` | `dmmsy::dmmsy()` | Main entry point |
| `CSRGraph` struct | `graph::CsrGraph` | CSR representation |
| `Fast4AryHeap` | `heap::FourAryHeap` | Cache-friendly priority queue |
| `malloc`/`free` | `Vec` | Pre-allocated workspace pattern |
| Raw pointer arithmetic | Slice indexing | `debug_assert!` for bounds |
| `random_graph()` | Dev-dependency rand | For testing only |

### Key Porting Considerations

1. **Replace malloc/free with Vec** — Pre-allocate in Workspace, pass `&mut`
2. **Replace raw pointers with slices** — Add `debug_assert!` for bounds
3. **The 4-ary heap is critical** — Binary heap has 2x more cache misses
4. **Frontier reduction is the core loop** — This is where DMMSY diverges from Dijkstra
5. **Zero-allocation subproblem decomposition** — Reuse workspace buffers across recursion

### Implementation Status

All algorithm stages are implemented, adapted from danalec's optimized
`ssp_duan` + `bmsp_rec` in `dmmsy_opt.rs`:

- [x] CsrGraph with CSR layout + mean_weight
- [x] Weight trait (generic f32/f64/u32/u64)
- [x] 1-indexed 4-ary heap (matching danalec's `Fast4AryHeap`)
- [x] Dijkstra baseline (correctness oracle, generic weights)
- [x] Auto-algorithm selection (`shortest_paths_f64`)
- [x] Workspace pre-allocation with dirty tracking
- [x] `ssp_duan` entry point with bound computation
- [x] `bmsp_rec` recursive BMSSP decomposition
- [x] Pivot selection by striding through source list
- [x] Bounded Dijkstra (only enqueue if `nd < b`)
- [x] Base case: full Dijkstra scan at max depth or few sources
- [x] Correctness verified: matches Dijkstra on random 1000-node graphs
