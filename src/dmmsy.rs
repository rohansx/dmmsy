//! Core DMMSY shortest path algorithm.
//!
//! Adapted from danalec's [DMMSY-SSSP-rs](https://github.com/danalec/DMMSY-SSSP-rs)
//! implementation of the algorithm from "Breaking the Sorting Barrier for
//! Directed Single-Source Shortest Paths" (Duan, Mao, Mao, Shu, Yin, STOC 2025).
//!
//! The key insight: Dijkstra maintains a TOTAL ordering of vertices by
//! distance (requires sorting → O(n log n)). DMMSY maintains only a
//! PARTIAL ordering using "pivot" vertices that split the problem into
//! independent subproblems, achieving O(m · log^(2/3) n).

use crate::dijkstra::ShortestPaths;
use crate::graph::CsrGraph;
use crate::heap::Fast4AryHeap;

const WEIGHT_MAX: f64 = f64::INFINITY;
const NODE_MAX: u32 = u32::MAX;

/// Algorithm parameters derived from graph size.
struct Params {
    k: u32,
    t: u32,
}

/// Compute algorithm parameters from node count.
///
/// - `k = max(4, floor(log2(n)^(1/3)))` — number of pivots per level
/// - `t = max(2, floor(log2(n)^(2/3)))` — max recursion depth
fn get_params(n: u32) -> Params {
    let log2n = (n as f64).log2();
    Params {
        k: 4.0_f64.max(log2n.powf(1.0 / 3.0).floor()) as u32,
        t: 2.0_f64.max(log2n.powf(2.0 / 3.0).floor()) as u32,
    }
}

/// Pre-allocated workspace for the DMMSY algorithm.
///
/// All scratch buffers are allocated once and reused. The `prepare()` method
/// efficiently resets only touched entries via dirty tracking.
struct Workspace {
    d: Vec<f64>,
    pr: Vec<u32>,
    h_nodes: Vec<crate::heap::HeapNode>,
    h_pos: Vec<u32>,
    dirty_h: Vec<u32>,
    dh_cnt: u32,
    dirty_d: Vec<u32>,
    ds_cnt: u32,
    piv_bufs: Vec<Vec<u32>>,
    #[allow(dead_code)]
    n: u32,
}

impl Workspace {
    fn new(n: u32, p: &Params) -> Self {
        let max_depth = p.t + 2;
        let buf_size = if p.k > 4 { p.k } else { 4 } as usize;

        Workspace {
            d: vec![WEIGHT_MAX; n as usize],
            pr: vec![NODE_MAX; n as usize],
            h_nodes: vec![crate::heap::HeapNode { v: 0.0, i: 0 }; (n + 1) as usize],
            h_pos: vec![0; n as usize],
            dirty_h: vec![0; (n + 1) as usize],
            dh_cnt: 0,
            dirty_d: vec![0; n as usize],
            ds_cnt: 0,
            piv_bufs: vec![vec![0u32; buf_size]; max_depth as usize],
            n,
        }
    }

    #[allow(dead_code)]
    fn prepare(&mut self, n: u32, p: &Params) {
        if self.n != n {
            *self = Self::new(n, p);
        } else {
            if self.ds_cnt > (self.n >> 2) {
                self.d.fill(WEIGHT_MAX);
                self.pr.fill(NODE_MAX);
            } else {
                for i in 0..self.ds_cnt {
                    let idx = self.dirty_d[i as usize] as usize;
                    self.d[idx] = WEIGHT_MAX;
                    self.pr[idx] = NODE_MAX;
                }
            }
            self.ds_cnt = 0;
            self.dh_cnt = 0;
        }
    }
}

/// Compute single-source shortest paths using the DMMSY algorithm.
///
/// Wraps danalec's optimized `ssp_duan` implementation with a clean API.
/// Operates on f64 weights (the algorithm uses floating-point distance
/// bounds internally).
///
/// For generic weight types, use [`shortest_paths`](crate::shortest_paths)
/// which auto-selects the best algorithm.
///
/// # Panics
///
/// Panics if `source >= graph.num_nodes()` or graph has 0 nodes.
pub fn dmmsy(graph: &CsrGraph<f64>, source: u32) -> ShortestPaths<f64> {
    let n = graph.num_nodes();
    assert!(n > 0, "cannot run DMMSY on empty graph");
    assert!((source as usize) < n, "source {} >= num_nodes {}", source, n);

    let n32 = n as u32;
    let p = get_params(n32);
    let mut ws = Workspace::new(n32, &p);

    // Initialize source
    ws.ds_cnt = 1;
    ws.d[source as usize] = 0.0;
    ws.dirty_d[0] = source;
    ws.dh_cnt = 0;

    // Compute distance bound from mean edge weight
    let log2_n1 = ((n + 1) as f64).log2();
    let b = graph.mean_weight() * log2_n1 * 4.0;

    ws.piv_bufs[1][0] = source;

    bmsp_rec(graph, 1, 0, 1, b, 0, &mut ws, &p);

    ShortestPaths {
        distances: ws.d,
        predecessors: ws.pr,
    }
}

/// Recursive bounded multi-source shortest path — the heart of DMMSY.
///
/// Recursively decomposes the problem:
/// 1. Select `k` pivots by striding through source list
/// 2. Recurse on pivots with halved distance bound
/// 3. Run bounded Dijkstra from all sources within bound `b`
///
/// Base case: at max recursion depth or few sources → full Dijkstra scan.
///
/// Adapted from danalec's `bmsp_rec` in `dmmsy_opt.rs`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn bmsp_rec(
    g: &CsrGraph<f64>,
    src_buf_lvl: usize,
    off_src: u32,
    len_src: u32,
    b: f64,
    dp: u32,
    ws: &mut Workspace,
    p: &Params,
) {
    // Base case: max depth or few enough sources → Dijkstra scan
    if dp >= p.t || len_src <= p.k {
        if ws.dh_cnt > 0 {
            for i in 0..ws.dh_cnt {
                ws.h_pos[ws.dirty_h[i as usize] as usize] = 0;
            }
            ws.dh_cnt = 0;
        }

        let mut h = Fast4AryHeap {
            nodes: std::mem::take(&mut ws.h_nodes),
            pos: std::mem::take(&mut ws.h_pos),
            dirty: std::mem::take(&mut ws.dirty_h),
        };

        let mut sz = 0u32;
        let mut dcnt = 0u32;

        for i in 0..len_src {
            let s = ws.piv_bufs[src_buf_lvl][(off_src + i) as usize];
            h.push_dec(&mut sz, &mut dcnt, s, ws.d[s as usize]);
        }
        ws.dh_cnt = dcnt;

        while sz > 0 {
            let mut du = 0.0;
            let mut u = 0u32;
            h.pop_min(&mut sz, &mut du, &mut u);

            if du > ws.d[u as usize] {
                continue;
            }

            let u_off = g.offset[u as usize];
            let u_end = g.offset[(u + 1) as usize];

            for i in u_off..u_end {
                let e = &g.edges[i as usize];
                let nd = du + e.w;

                if nd < ws.d[e.v as usize] {
                    if ws.d[e.v as usize] == WEIGHT_MAX {
                        ws.dirty_d[ws.ds_cnt as usize] = e.v;
                        ws.ds_cnt += 1;
                    }
                    ws.d[e.v as usize] = nd;
                    ws.pr[e.v as usize] = u;
                    h.push_dec(&mut sz, &mut ws.dh_cnt, e.v, nd);
                }
            }
        }

        ws.h_nodes = h.nodes;
        ws.h_pos = h.pos;
        ws.dirty_h = h.dirty;
        return;
    }

    // Recursive case: select pivots, recurse with halved bound
    let np = if len_src < p.k { len_src } else { p.k };
    let mut step = len_src / np;
    if step == 0 {
        step = 1;
    }

    let bound = if len_src < (step * p.k) { len_src } else { step * p.k };
    let next_buf_lvl = (dp + 2) as usize;

    let mut curr_np = 0u32;
    let mut i = 0u32;
    while i < bound {
        let pivot_val = ws.piv_bufs[src_buf_lvl][(off_src + i) as usize];
        ws.piv_bufs[next_buf_lvl][curr_np as usize] = pivot_val;
        curr_np += 1;
        i += step;
    }

    // Phase 1: Recurse on pivots with halved distance bound
    bmsp_rec(g, next_buf_lvl, 0, curr_np, b * 0.5, dp + 1, ws, p);

    // Phase 2: Bounded Dijkstra from all sources within bound b
    if ws.dh_cnt > 0 {
        for j in 0..ws.dh_cnt {
            ws.h_pos[ws.dirty_h[j as usize] as usize] = 0;
        }
        ws.dh_cnt = 0;
    }

    let mut h = Fast4AryHeap {
        nodes: std::mem::take(&mut ws.h_nodes),
        pos: std::mem::take(&mut ws.h_pos),
        dirty: std::mem::take(&mut ws.dirty_h),
    };

    let mut sz = 0u32;
    let mut dcnt = 0u32;
    let mut has_work = false;

    for j in 0..len_src {
        let s = ws.piv_bufs[src_buf_lvl][(off_src + j) as usize];
        let dv = ws.d[s as usize];
        if dv < b {
            h.push_dec(&mut sz, &mut dcnt, s, dv);
            has_work = true;
        }
    }
    ws.dh_cnt = dcnt;

    if !has_work {
        ws.h_nodes = h.nodes;
        ws.h_pos = h.pos;
        ws.dirty_h = h.dirty;
        return;
    }

    // Bounded Dijkstra: only enqueue if nd < b
    while sz > 0 {
        let mut du = 0.0;
        let mut u = 0u32;
        h.pop_min(&mut sz, &mut du, &mut u);

        if du > ws.d[u as usize] {
            continue;
        }

        let u_off = g.offset[u as usize];
        let u_end = g.offset[(u + 1) as usize];

        for i in u_off..u_end {
            let e = &g.edges[i as usize];
            let nd = du + e.w;

            if nd < ws.d[e.v as usize] {
                if ws.d[e.v as usize] == WEIGHT_MAX {
                    ws.dirty_d[ws.ds_cnt as usize] = e.v;
                    ws.ds_cnt += 1;
                }
                ws.d[e.v as usize] = nd;
                ws.pr[e.v as usize] = u;
                if nd < b {
                    h.push_dec(&mut sz, &mut ws.dh_cnt, e.v, nd);
                }
            }
        }
    }

    ws.h_nodes = h.nodes;
    ws.h_pos = h.pos;
    ws.dirty_h = h.dirty;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dijkstra;

    fn assert_matches_dijkstra(g: &CsrGraph<f64>, source: u32) {
        let dmmsy_result = dmmsy(g, source);
        let dijkstra_result = dijkstra::dijkstra(g, source);
        for i in 0..g.num_nodes() {
            let dd = dmmsy_result.distances[i];
            let dj = dijkstra_result.distances[i];
            if dd.is_infinite() && dj.is_infinite() {
                continue;
            }
            assert!(
                (dd - dj).abs() < 1e-6,
                "mismatch at node {}: dmmsy={}, dijkstra={}", i, dd, dj
            );
        }
    }

    #[test]
    fn simple_path() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 1.0), (1, 2, 2.0), (0, 2, 5.0), (2, 3, 1.0),
        ]);
        assert_matches_dijkstra(&g, 0);
    }

    #[test]
    fn diamond() {
        let g = CsrGraph::from_edges(4, &[
            (0, 1, 2.0), (0, 2, 3.0), (1, 3, 1.0), (2, 3, 1.0),
        ]);
        assert_matches_dijkstra(&g, 0);
    }

    #[test]
    fn disconnected() {
        let g = CsrGraph::from_edges(5, &[
            (0, 1, 1.0), (1, 2, 2.0), (3, 4, 1.0),
        ]);
        assert_matches_dijkstra(&g, 0);
    }

    #[test]
    fn single_node() {
        let g = CsrGraph::from_edges(1, &[]);
        let result = dmmsy(&g, 0);
        assert_eq!(result.distances[0], 0.0);
    }

    #[test]
    fn random_1000() {
        use rand::Rng;
        use rand_xoshiro::rand_core::SeedableRng;
        use rand_xoshiro::Xoshiro256StarStar;

        let mut rng = Xoshiro256StarStar::seed_from_u64(42);
        let n = 1000;
        let m = 5000;
        let mut edges = Vec::with_capacity(m);
        for _ in 0..m {
            let src = rng.gen_range(0..n as u32);
            let tgt = rng.gen_range(0..n as u32);
            let w: f64 = rng.gen_range(0.1..100.0);
            edges.push((src, tgt, w));
        }
        let g = CsrGraph::from_edges(n, &edges);
        assert_matches_dijkstra(&g, 0);
    }
}
