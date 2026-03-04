#!/bin/bash
# dmmsy demo — SSSP library crate
set -e

echo "═══════════════════════════════════════════════════"
echo "  dmmsy — Usable DMMSY Shortest Path Library"
echo "═══════════════════════════════════════════════════"
echo ""
sleep 1

echo ">> Detecting project..."
echo "   Based on the STOC 2025 paper by Duan, Mao, Mao, Shu, Yin"
echo "   Wraps danalec's Rust implementation as a proper library crate"
echo ""
sleep 2

echo ">> Running tests (28 unit + 8 doc)..."
cargo test --quiet 2>&1
echo ""
sleep 1

echo ">> Running the basic example..."
echo "   (Creates a graph, computes shortest paths)"
echo ""
cargo run --example basic --quiet 2>&1
echo ""
sleep 2

echo ">> Running the road network example..."
echo "   (Simulated road network with u32 weights)"
echo ""
cargo run --example road_network --quiet 2>&1
echo ""
sleep 2

echo ">> Running benchmarks (sparse 10K nodes)..."
cargo bench -- "sparse-random/shortest_paths/10000" --quiet 2>&1 | grep -E "(time:|thrpt:)" | head -2
cargo bench -- "sparse-random/dijkstra/10000" --quiet 2>&1 | grep -E "(time:|thrpt:)" | head -2
echo ""
sleep 1

echo ">> Key features:"
echo "   - Generic Weight trait (f64, f32, u32, u64)"
echo "   - Auto-selects DMMSY vs Dijkstra based on graph density"
echo "   - petgraph integration via DmmsyExt trait"
echo "   - DIMACS .gr format parser"
echo "   - Zero unsafe code in public API"
echo ""
echo ">> cargo add dmmsy  # coming soon to crates.io"
echo ""
echo "═══════════════════════════════════════════════════"
