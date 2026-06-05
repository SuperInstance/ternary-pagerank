//! # ternary-pagerank
//!
//! PageRank and centrality measures on ternary-weighted graphs.
//! Importance propagation, authority scoring, and community detection.

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{vec, vec::Vec};

/// A ternary-weighted directed graph
#[derive(Debug, Clone)]
pub struct TernaryGraph {
    pub n: usize,
    /// adjacency[i][j] = weight of edge from i to j (-1, 0, or 1)
    pub adjacency: Vec<Vec<i8>>,
}

impl TernaryGraph {
    pub fn new(n: usize) -> Self {
        Self { n, adjacency: vec![vec![0i8; n]; n] }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: i8) {
        self.adjacency[from][to] = weight.clamp(-1, 1);
    }

    pub fn out_degree(&self, node: usize) -> i8 {
        self.adjacency[node].iter().filter(|&&w| w != 0).count() as i8
    }

    pub fn in_degree(&self, node: usize) -> i8 {
        (0..self.n).filter(|&i| self.adjacency[i][node] != 0).count() as i8
    }

    /// PageRank iteration: scores propagate along edges weighted by adjacency
    pub fn pagerank_step(&self, scores: &[i8], damping: i8) -> Vec<i8> {
        let n = self.n;
        let mut new_scores = vec![0i8; n];

        // Random surfer contribution: (1-damping)/n
        let base = match damping {
            1 => 0,
            _ => 1,
            // In ternary: simplified damping
        };

        for j in 0..n {
            let mut rank = base;
            for i in 0..n {
                let weight = self.adjacency[i][j];
                if weight != 0 {
                    let out = self.out_degree(i);
                    if out > 0 {
                        // rank += score[i] * weight / out_degree[i]
                        // Simplified: just propagate ternary values
                        rank += scores[i] * weight / out;
                    }
                }
            }
            new_scores[j] = rank.clamp(-1, 1);
        }

        new_scores
    }

    /// Run PageRank until convergence (or max iterations)
    pub fn pagerank(&self, max_iterations: usize) -> Vec<i8> {
        let mut scores = vec![1i8; self.n]; // start with all nodes important
        for _ in 0..max_iterations {
            let new_scores = self.pagerank_step(&scores, 1);
            if new_scores == scores {
                break;
            }
            scores = new_scores;
        }
        scores
    }

    /// Betweenness centrality: how many shortest paths pass through this node?
    /// Simplified for ternary: count paths of length 2 that go through each node
    pub fn betweenness(&self) -> Vec<i8> {
        let mut centrality = vec![0i8; self.n];
        for s in 0..self.n {
            for t in 0..self.n {
                if s == t { continue; }
                // Find intermediate nodes v such that s→v and v→t both exist
                for v in 0..self.n {
                    if v == s || v == t { continue; }
                    if self.adjacency[s][v] != 0 && self.adjacency[v][t] != 0 {
                        centrality[v] = (centrality[v] + 1).min(1);
                    }
                }
            }
        }
        centrality
    }

    /// Eigenvector centrality (simplified): one node's score = sum of neighbor scores
    pub fn eigenvector_step(&self, scores: &[i8]) -> Vec<i8> {
        let mut new_scores = vec![0i8; self.n];
        for j in 0..self.n {
            let mut sum = 0i8;
            for i in 0..self.n {
                if self.adjacency[i][j] != 0 {
                    sum += scores[i];
                }
            }
            new_scores[j] = sum.clamp(-1, 1);
        }
        new_scores
    }

    /// Run eigenvector centrality to convergence
    pub fn eigenvector_centrality(&self, max_iterations: usize) -> Vec<i8> {
        let mut scores = vec![0i8; self.n];
        for i in 0..self.n {
            scores[i] = if self.in_degree(i) > 0 { 1 } else { 0 };
        }
        for _ in 0..max_iterations {
            let new_scores = self.eigenvector_step(&scores);
            if new_scores == scores { break; }
            scores = new_scores;
        }
        scores
    }

    /// Community detection: find connected components based on positive-weight edges
    pub fn communities(&self) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.n];
        let mut components = vec![];

        for start in 0..self.n {
            if visited[start] { continue; }
            let mut component = vec![];
            let mut stack = vec![start];
            while let Some(node) = stack.pop() {
                if visited[node] { continue; }
                visited[node] = true;
                component.push(node);
                for neighbor in 0..self.n {
                    if !visited[neighbor] && self.adjacency[node][neighbor] > 0 {
                        stack.push(neighbor);
                    }
                }
            }
            components.push(component);
        }

        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let g = TernaryGraph::new(3);
        assert_eq!(g.out_degree(0), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        assert_eq!(g.out_degree(0), 1);
        assert_eq!(g.in_degree(1), 1);
    }

    #[test]
    fn test_negative_edge() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, -1);
        assert_eq!(g.adjacency[0][1], -1);
    }

    #[test]
    fn test_pagerank_simple() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.add_edge(2, 0, 1);
        let scores = g.pagerank(20);
        assert_eq!(scores.len(), 3);
    }

    #[test]
    fn test_pagerank_hub() {
        let mut g = TernaryGraph::new(4);
        // Node 0 links to all others
        g.add_edge(0, 1, 1);
        g.add_edge(0, 2, 1);
        g.add_edge(0, 3, 1);
        // Add back-links so rank flows
        g.add_edge(1, 0, 1);
        g.add_edge(2, 0, 1);
        g.add_edge(3, 0, 1);
        let scores = g.pagerank(20);
        // With bidirectional links, scores should propagate
        assert_eq!(scores.len(), 4);
    }

    #[test]
    fn test_betweenness() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        let bc = g.betweenness();
        // Node 1 is on the path between 0 and 2
        assert_eq!(bc[1], 1);
        assert_eq!(bc[0], 0);
    }

    #[test]
    fn test_eigenvector_step() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        let scores = vec![1, 0, 0];
        let new_scores = g.eigenvector_step(&scores);
        // Node 1 gets score from node 0
        assert_eq!(new_scores[1], 1);
    }

    #[test]
    fn test_eigenvector_centrality() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.add_edge(2, 0, 1);
        let scores = g.eigenvector_centrality(20);
        assert_eq!(scores.len(), 3);
    }

    #[test]
    fn test_communities_connected() {
        let mut g = TernaryGraph::new(4);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, 1);
        g.add_edge(2, 3, 1);
        let communities = g.communities();
        assert_eq!(communities.len(), 1); // all connected
    }

    #[test]
    fn test_communities_disconnected() {
        let mut g = TernaryGraph::new(4);
        g.add_edge(0, 1, 1);
        g.add_edge(2, 3, 1);
        let communities = g.communities();
        assert_eq!(communities.len(), 2);
    }

    #[test]
    fn test_communities_negative_edge_excluded() {
        let mut g = TernaryGraph::new(3);
        g.add_edge(0, 1, 1);
        g.add_edge(1, 2, -1); // negative edge doesn't connect communities
        let communities = g.communities();
        assert_eq!(communities.len(), 2); // 0-1 are one, 2 is alone
    }
}
