//! Cache-friendly 4-ary min-heap for priority queue operations.
//!
//! Adapted from danalec's DMMSY-SSSP-rs implementation. Uses 1-indexed
//! storage and tracks dirty entries for efficient workspace reuse across
//! recursive calls — critical for DMMSY's performance.
//!
//! A 4-ary heap stores 4 children per node. Children of node `i` are at
//! indices `4i-2..4i+2` (1-indexed), which typically fit in a single
//! cache line. This halves cache misses compared to a binary heap.

/// Heap entry: (priority, node_id).
#[derive(Clone, Copy, Debug)]
pub struct HeapNode {
    /// Priority value (distance).
    pub v: f64,
    /// Node identifier.
    pub i: u32,
}

/// A 1-indexed 4-ary min-heap with decrease-key support.
///
/// Tracks "dirty" entries for efficient cleanup between recursive calls
/// in the DMMSY algorithm. The `pos` array maps node → heap position,
/// and `dirty` tracks which nodes have been touched.
pub struct Fast4AryHeap {
    /// Heap storage (1-indexed, index 0 unused).
    pub nodes: Vec<HeapNode>,
    /// Position map: `pos[node] = heap_index`, 0 = not in heap, MAX = popped.
    pub pos: Vec<u32>,
    /// Dirty node list for efficient cleanup.
    pub dirty: Vec<u32>,
}

const NODE_MAX: u32 = u32::MAX;

impl Fast4AryHeap {
    /// Create a new heap with capacity for `n` nodes.
    pub fn new(n: u32) -> Self {
        Self {
            nodes: vec![HeapNode { v: 0.0, i: 0 }; (n + 1) as usize],
            pos: vec![0; n as usize],
            dirty: vec![0; n as usize],
        }
    }

    /// Sift element at `i` upward to restore heap property.
    pub fn push_up(&mut self, mut i: u32) {
        let node = self.nodes[i as usize];
        while i > 1 {
            let par = (i - 2) / 4 + 1;
            let pnode = self.nodes[par as usize];
            if pnode.v <= node.v {
                break;
            }
            self.nodes[i as usize] = pnode;
            self.pos[pnode.i as usize] = i;
            i = par;
        }
        self.nodes[i as usize] = node;
        self.pos[node.i as usize] = i;
    }

    /// Sift element at `i` downward to restore heap property.
    pub fn push_down(&mut self, mut i: u32, sz: u32) {
        let node = self.nodes[i as usize];
        loop {
            let c1 = (i * 4).wrapping_sub(2);
            if c1 > sz {
                break;
            }

            let mut mc = c1;
            let mut mc_node = self.nodes[c1 as usize];
            let mut mcv = mc_node.v;

            if c1 < sz && self.nodes[c1 as usize + 1].v < mcv {
                mc = c1 + 1;
                mc_node = self.nodes[mc as usize];
                mcv = mc_node.v;
            }
            if c1 + 2 <= sz && self.nodes[(c1 + 2) as usize].v < mcv {
                mc = c1 + 2;
                mc_node = self.nodes[mc as usize];
                mcv = mc_node.v;
            }
            if c1 + 3 <= sz && self.nodes[(c1 + 3) as usize].v < mcv {
                mc = c1 + 3;
                mc_node = self.nodes[mc as usize];
                // mcv not needed after this
                let _ = mc_node;
            }

            if node.v <= self.nodes[mc as usize].v {
                break;
            }

            let mc_node = self.nodes[mc as usize];
            self.nodes[i as usize] = mc_node;
            self.pos[mc_node.i as usize] = i;
            i = mc;
        }
        self.nodes[i as usize] = node;
        self.pos[node.i as usize] = i;
    }

    /// Insert or decrease-key. Updates `sz` (heap size) and `dcnt` (dirty count).
    ///
    /// If node `n` is not in the heap, inserts it. If already in the heap
    /// with a higher priority, decreases it. Otherwise does nothing.
    pub fn push_dec(&mut self, sz: &mut u32, dcnt: &mut u32, n: u32, d: f64) {
        let p = self.pos[n as usize];
        let i;
        if p == 0 || p == NODE_MAX {
            // Not in heap — insert
            *sz += 1;
            *dcnt += 1;
            i = *sz;
            if (*dcnt - 1) < self.dirty.len() as u32 {
                self.dirty[(*dcnt - 1) as usize] = n;
            }
        } else {
            // Already in heap — try decrease
            i = p;
            if d >= self.nodes[i as usize].v {
                return;
            }
        }

        self.nodes[i as usize].v = d;
        self.nodes[i as usize].i = n;
        self.push_up(i);
    }

    /// Remove and return the minimum element.
    ///
    /// Sets `mv` to the minimum priority and `mn` to the minimum node ID.
    pub fn pop_min(&mut self, sz: &mut u32, mv: &mut f64, mn: &mut u32) {
        let min_node = self.nodes[1];
        *mv = min_node.v;
        *mn = min_node.i;

        self.pos[*mn as usize] = NODE_MAX;

        if *sz == 1 {
            *sz -= 1;
            return;
        }

        let last_node = self.nodes[*sz as usize];
        *sz -= 1;

        self.nodes[1] = last_node;
        self.pos[last_node.i as usize] = 1;
        self.push_down(1, *sz);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_pop_ordered() {
        let mut h = Fast4AryHeap::new(10);
        let mut sz = 0u32;
        let mut dcnt = 0u32;

        h.push_dec(&mut sz, &mut dcnt, 3, 3.0);
        h.push_dec(&mut sz, &mut dcnt, 1, 1.0);
        h.push_dec(&mut sz, &mut dcnt, 2, 2.0);

        assert_eq!(sz, 3);

        let (mut mv, mut mn) = (0.0, 0u32);
        h.pop_min(&mut sz, &mut mv, &mut mn);
        assert_eq!(mn, 1);
        assert_eq!(mv, 1.0);

        h.pop_min(&mut sz, &mut mv, &mut mn);
        assert_eq!(mn, 2);

        h.pop_min(&mut sz, &mut mv, &mut mn);
        assert_eq!(mn, 3);

        assert_eq!(sz, 0);
    }

    #[test]
    fn decrease_key() {
        let mut h = Fast4AryHeap::new(10);
        let mut sz = 0u32;
        let mut dcnt = 0u32;

        h.push_dec(&mut sz, &mut dcnt, 0, 10.0);
        h.push_dec(&mut sz, &mut dcnt, 1, 5.0);
        // Decrease node 0 from 10.0 to 1.0
        h.push_dec(&mut sz, &mut dcnt, 0, 1.0);

        let (mut mv, mut mn) = (0.0, 0u32);
        h.pop_min(&mut sz, &mut mv, &mut mn);
        assert_eq!(mn, 0);
        assert_eq!(mv, 1.0);
    }

    #[test]
    fn many_elements() {
        let n = 100u32;
        let mut h = Fast4AryHeap::new(n);
        let mut sz = 0u32;
        let mut dcnt = 0u32;

        for i in (0..n).rev() {
            h.push_dec(&mut sz, &mut dcnt, i, i as f64);
        }

        let (mut mv, mut mn) = (0.0, 0u32);
        for expected in 0..n {
            h.pop_min(&mut sz, &mut mv, &mut mn);
            assert_eq!(mn, expected);
        }
    }
}
