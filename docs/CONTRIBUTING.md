# Contributing

## Development Setup

```sh
git clone https://github.com/rohanxyz/dmmsy
cd dmmsy
cargo build
cargo test
```

## Code Quality

Before submitting:

```sh
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

## Testing

### Unit Tests

Every module has tests. Run them all:

```sh
cargo test
```

### Correctness Invariant

The fundamental invariant: **DMMSY must produce identical distances to Dijkstra on every input.**

Any test that generates a random graph should run both algorithms and compare results:

```rust
let dmmsy_result = dmmsy::dmmsy(&g, 0);
let dijkstra_result = dijkstra::dijkstra(&g, 0);
for i in 0..n {
    assert_eq!(dmmsy_result.distances[i], dijkstra_result.distances[i]);
}
```

### Adding Tests

- Small graph tests: use hand-verified expected values
- Large graph tests: use the DMMSY-vs-Dijkstra invariant
- Edge cases: disconnected components, self-loops, zero-weight edges, single node
- Use `Xoshiro256StarStar::seed_from_u64(42)` for reproducible randomness

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for module layout and design.
See [DESIGN.md](DESIGN.md) for rationale behind key decisions.
See [ALGORITHM.md](ALGORITHM.md) for the DMMSY algorithm details.

## Implementation Priorities

Current focus: completing the full DMMSY algorithm port.

1. Pivot selection (from C99 reference)
2. Frontier reduction
3. Recursive BMSSP decomposition
4. Block-based linked lists
5. Partial-order priority queue

Each stage should be implemented as a separate PR with:
- The new code
- Tests verifying correctness (DMMSY == Dijkstra invariant)
- Benchmarks showing any performance change
