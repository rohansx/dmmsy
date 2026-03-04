//! DIMACS graph format parser.
//!
//! Parses `.gr` files from the [9th DIMACS Implementation Challenge](http://www.dis.uniroma1.it/challenge9/)
//! for shortest path benchmarking. Standard road network benchmark data.
//!
//! # Format
//!
//! ```text
//! c Comment lines start with 'c'
//! p sp <num_nodes> <num_edges>
//! a <source> <target> <weight>
//! a <source> <target> <weight>
//! ...
//! ```
//!
//! Node IDs in DIMACS are 1-indexed; this parser converts to 0-indexed.

use crate::graph::CsrGraph;
use std::fs;
use std::io;
use std::path::Path;

/// Parse a DIMACS `.gr` file into a [`CsrGraph<u32>`].
///
/// DIMACS road networks use integer edge weights (travel time or distance).
///
/// # Errors
///
/// Returns `io::Error` if the file cannot be read or has invalid format.
///
/// # Example
///
/// ```no_run
/// use dmmsy::dimacs::from_file;
///
/// let graph = from_file("data/USA-road-d.NY.gr").unwrap();
/// println!("Nodes: {}, Edges: {}", graph.num_nodes(), graph.num_edges());
/// ```
pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<CsrGraph<u32>> {
    let contents = fs::read_to_string(path)?;
    from_str(&contents)
}

/// Parse a DIMACS `.gr` format string into a [`CsrGraph<u32>`].
pub fn from_str(input: &str) -> io::Result<CsrGraph<u32>> {
    let mut num_nodes = 0;
    let mut edges = Vec::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('c') {
            continue;
        }

        if line.starts_with("p sp") || line.starts_with("p SP") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid problem line",
                ));
            }
            num_nodes = parts[2].parse::<usize>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
            let num_edges = parts[3].parse::<usize>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
            edges.reserve(num_edges);
        } else if line.starts_with('a') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid arc line: {}", line),
                ));
            }
            // DIMACS is 1-indexed, convert to 0-indexed
            let src = parts[1].parse::<u32>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })? - 1;
            let tgt = parts[2].parse::<u32>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })? - 1;
            let weight = parts[3].parse::<u32>().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e)
            })?;
            edges.push((src, tgt, weight));
        }
    }

    if num_nodes == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "no problem line found",
        ));
    }

    Ok(CsrGraph::from_edges(num_nodes, &edges))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_small_dimacs() {
        let input = "\
c Example DIMACS graph
c 4 nodes, 5 edges
p sp 4 5
a 1 2 10
a 1 3 5
a 2 3 2
a 3 4 1
a 2 4 8
";
        let g = from_str(input).unwrap();
        assert_eq!(g.num_nodes(), 4);
        assert_eq!(g.num_edges(), 5);

        // Node 0 (was 1) should have edges to nodes 1 and 2
        let n0: Vec<_> = g.neighbors(0).collect();
        assert_eq!(n0.len(), 2);

        // Node 2 (was 3) should have edge to node 3
        let n2: Vec<_> = g.neighbors(2).collect();
        assert_eq!(n2.len(), 1);
        assert_eq!(n2[0], (3, 1));
    }

    #[test]
    fn empty_comments_only() {
        let input = "\
c Just comments
c No problem line
";
        assert!(from_str(input).is_err());
    }

    #[test]
    fn single_edge() {
        let input = "\
p sp 2 1
a 1 2 42
";
        let g = from_str(input).unwrap();
        assert_eq!(g.num_nodes(), 2);
        assert_eq!(g.num_edges(), 1);
        let n: Vec<_> = g.neighbors(0).collect();
        assert_eq!(n, vec![(1, 42u32)]);
    }
}
