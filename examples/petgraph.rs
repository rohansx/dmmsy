//! Using dmmsy with petgraph types.
//!
//! Requires the `petgraph` feature:
//! ```sh
//! cargo run --example petgraph --features petgraph
//! ```

#[cfg(feature = "petgraph")]
fn main() {
    use petgraph::graph::DiGraph;
    use dmmsy::petgraph_compat::DmmsyExt;

    let mut g = DiGraph::new();
    let home = g.add_node("Home");
    let work = g.add_node("Work");
    let store = g.add_node("Store");
    let gym = g.add_node("Gym");
    let park = g.add_node("Park");

    g.add_edge(home, work, 10.0_f64);
    g.add_edge(home, store, 3.0);
    g.add_edge(store, work, 5.0);
    g.add_edge(work, gym, 2.0);
    g.add_edge(store, gym, 12.0);
    g.add_edge(gym, park, 4.0);
    g.add_edge(work, park, 8.0);

    let paths = g.dmmsy_shortest_paths(home);

    println!("Shortest paths from Home:");
    for node in [home, work, store, gym, park] {
        let idx = node.index() as u32;
        let name = g[node];
        println!("  → {}: {:.1}", name, paths.distance(idx));
    }

    let park_idx = park.index() as u32;
    if let Some(path) = paths.path_to(park_idx) {
        let names: Vec<&str> = path.iter()
            .map(|&n| g[petgraph::graph::NodeIndex::new(n as usize)])
            .collect();
        println!("\nShortest path Home → Park: {}", names.join(" → "));
    }
}

#[cfg(not(feature = "petgraph"))]
fn main() {
    eprintln!("This example requires the `petgraph` feature.");
    eprintln!("Run with: cargo run --example petgraph --features petgraph");
}
