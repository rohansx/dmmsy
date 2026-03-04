# Design Decisions

## Why CSR Instead of Adjacency List

**Decision:** Use Compressed Sparse Row (CSR) format instead of `Vec<Vec<(u32, W)>>`.

**Reasoning:**
- CSR stores all edge data in contiguous arrays → sequential memory access
- Adjacency lists scatter edge data across heap → random memory access
- Edge relaxation (iterating neighbors) is the hot loop — cache performance dominates
- CSR uses less memory: no per-node Vec overhead (24 bytes × n saved)
- Trade-off: CSR is immutable after construction. This is fine for SSSP (graph doesn't change during computation).

## Why 4-Ary Heap Instead of Binary Heap

**Decision:** Use a 4-ary min-heap instead of `BinaryHeap` from std.

**Reasoning:**
- Binary heap children at `2i+1`, `2i+2` — often span two cache lines
- 4-ary heap children at `4i+1..4i+4` — fit in one cache line
- ~2x fewer L1 cache misses on large graphs
- Needed `decrease_key` which std's BinaryHeap doesn't support
- Both DMMSY and Dijkstra share the same heap → fair algorithm comparison

## Why Generic Weight Trait

**Decision:** Define `trait Weight` instead of hardcoding `f64`.

**Reasoning:**
- DIMACS road networks use integer weights (u32) — no floating point needed
- Some applications need f32 (memory savings for billion-edge graphs)
- The trait is minimal: `Copy + PartialOrd + Add + INFINITY + ZERO`
- Zero runtime cost — monomorphized at compile time

## Why Auto-Selection Instead of Only DMMSY

**Decision:** `shortest_paths()` auto-selects between DMMSY and Dijkstra.

**Reasoning:**
- DMMSY has higher constant factors than Dijkstra
- On small or dense graphs, Dijkstra is faster despite worse asymptotics
- Users shouldn't need to know graph theory to pick the right algorithm
- Threshold: n < 1024 or density > 0.25 → Dijkstra
- Users can force either via `dmmsy()` or `dijkstra()` directly

## Why petgraph Integration as Optional Feature

**Decision:** petgraph support behind `features = ["petgraph"]`.

**Reasoning:**
- petgraph is a large dependency (pulls in fixedbitset, indexmap)
- Users with their own graph types shouldn't pay for it
- The integration is thin: `From<DiGraph>` + one extension trait
- Dev-dependencies always include petgraph (for testing against their Dijkstra)

## Why DIMACS Parser in the Crate

**Decision:** Include `dimacs.rs` for parsing .gr files.

**Reasoning:**
- DIMACS is the standard benchmark format for SSSP algorithms
- Without it, users can't easily reproduce benchmarks
- It's ~80 lines of code — not worth a separate crate
- Only depends on std::fs (no extra dependencies)

## Why ShortestPaths Has Both distances AND predecessors

**Decision:** Return both distance array and predecessor array.

**Reasoning:**
- Distances alone answer "how far?" but not "which path?"
- Predecessor array enables `path_to()` reconstruction
- Both are computed as a side-effect of the algorithm (zero extra cost)
- Users who don't need predecessors can ignore the field (it's still allocated, but the overhead is negligible)

## Future: Unsafe in Hot Loops

**Decision:** Currently no unsafe code. May add `get_unchecked()` in innermost loop behind a feature flag.

**Reasoning:**
- Bounds checking in the edge relaxation loop shows up in profiles on large graphs
- But safety is the default — we want correctness first
- If added, it will be behind `features = ["unchecked"]` and documented
- The scalar fallback (bounds-checked) will always exist for debugging
