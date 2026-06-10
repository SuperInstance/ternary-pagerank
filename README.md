# ternary-pagerank

**PageRank on graphs where every edge carries {-1, 0, +1} weight. Importance propagation, centrality scoring, and community detection — with the sign of the edge changing everything.**

## Why This Exists

Standard PageRank treats all edges as equal endorsements. A link from A to B means A vouches for B. But real networks aren't that simple. Social networks have trust and distrust. Financial networks have assets and liabilities. Neural networks have excitatory and inhibitory connections.

When you give edges ternary weights {-1, 0, +1}, PageRank becomes a different beast. A +1 edge propagates importance (endorsement). A -1 edge *drains* importance (opposition). A 0 edge is the absence of a relationship. The resulting importance scores aren't just "who's popular" — they're "who's popular with the right people and opposed by the wrong people."

This crate implements PageRank and three centrality measures (betweenness, eigenvector, and community detection) for ternary-weighted directed graphs. The `#![no_std]` design means it works in constrained environments.

## The Key Insight

In standard PageRank, a node's score flows equally to all its out-neighbors. With ternary weights, the flow is modulated:

- **+1 edge**: Importance flows from source to target (endorsement)
- **-1 edge**: Importance flows *negatively* — the target's score decreases (opposition)
- **No edge**: No flow

This creates dynamics that don't exist in unsigned PageRank. Consider three nodes where A→B (+1), B→C (+1), C→A (-1). C opposes A. In unsigned PageRank, all three nodes would have equal score (it's a cycle). In ternary PageRank, A's score gets *drained* by C's negative edge, while B benefits from A's endorsement. The steady state isn't uniform — it reflects the signed structure.

The community detection uses this directly: communities are connected components through *positive* edges only. A negative edge is a boundary, not a bridge. Two nodes connected only by a -1 edge are in different communities — they're in opposition, not agreement.

## Quick Start

```rust
use ternary_pagerank::*;

// Build a graph: 4 nodes
let mut g = TernaryGraph::new(4);
g.add_edge(0, 1, 1);   // A endorses B
g.add_edge(0, 2, 1);   // A endorses C
g.add_edge(0, 3, 1);   // A endorses D
g.add_edge(1, 0, 1);   // B endorses A back
g.add_edge(2, 0, 1);   // C endorses A back
g.add_edge(3, 0, 1);   // D endorses A back

// PageRank
let scores = g.pagerank(20);
assert_eq!(scores.len(), 4);

// Betweenness: who sits on the most paths?
let bc = g.betweenness();
// Node 0 is the hub, but in this symmetric graph, 
// betweenness might be low (everyone connects directly)

// Eigenvector centrality: who's connected to other important nodes?
let ec = g.eigenvector_centrality(20);

// Community detection: find connected components via positive edges
let communities = g.communities();
```

### Negative Edges Change Everything

```rust
let mut g = TernaryGraph::new(4);

// Two separate communities: {0,1} and {2,3}
g.add_edge(0, 1, 1);   // Within-group: positive
g.add_edge(1, 0, 1);
g.add_edge(2, 3, 1);   // Within-group: positive
g.add_edge(3, 2, 1);

// Cross-group: negative
g.add_edge(1, 2, -1);   // Opposition between groups
g.add_edge(2, 1, -1);

let communities = g.communities();
assert_eq!(communities.len(), 2);  // Two separate communities

// The negative edge doesn't connect them — it divides them
```

## Architecture

### Core Type: TernaryGraph

```rust
pub struct TernaryGraph {
    pub n: usize,                        // Number of nodes
    pub adjacency: Vec<Vec<i8>>,         // n×n matrix of {-1, 0, +1}
}
```

The adjacency matrix representation is chosen deliberately for ternary graphs. Sparse representations (edge lists) are more memory-efficient for large graphs, but the adjacency matrix makes the ternary structure explicit: `adjacency[i][j]` is the weight from i to j, and it's always in {-1, 0, +1}.

### Algorithms

| Method | Algorithm | Output |
|--------|-----------|--------|
| `pagerank` | Iterative propagation with ternary-weighted flow | `Vec<i8>` scores |
| `pagerank_step` | Single PageRank iteration | `Vec<i8>` |
| `betweenness` | Path-counting (length-2 paths through each node) | `Vec<i8>` |
| `eigenvector_centrality` | Power iteration on adjacency | `Vec<i8>` |
| `eigenvector_step` | Single power iteration | `Vec<i8>` |
| `communities` | DFS on positive-weight edges | `Vec<Vec<usize>>` |

### Ternary Scores

All centrality scores are ternary: {-1, 0, +1}. This isn't a limitation of the implementation — it's a feature. Ternary scores give you three meaningful categories:

- **+1**: High importance / central / community member
- **0**: Neutral / peripheral / disconnected
- **-1**: Anti-important / opposed / anti-community

The clamping to {-1, 0, +1} happens naturally from the Z₃ arithmetic. Intermediate values don't accumulate because the ternary operations wrap.

## API Reference

### TernaryGraph

| Method | Signature | Description |
|--------|-----------|-------------|
| `new(n)` | `usize → TernaryGraph` | Create n-node graph |
| `add_edge(from, to, w)` | Set edge weight to {-1, 0, +1} | |
| `out_degree(node)` | `→ i8` | Non-zero out-edges |
| `in_degree(node)` | `→ i8` | Non-zero in-edges |
| `pagerank(max_iter)` | `→ Vec<i8>` | Converged importance scores |
| `pagerank_step(scores, damping)` | `&[i8] → Vec<i8>` | One iteration |
| `betweenness()` | `→ Vec<i8>` | Betweenness centrality |
| `eigenvector_centrality(max_iter)` | `→ Vec<i8>` | Eigenvector centrality |
| `eigenvector_step(scores)` | `&[i8] → Vec<i8>` | One power iteration |
| `communities()` | `→ Vec<Vec<usize>>` | Connected components via +1 edges |

## Real-World Example: Trust Network

```rust
use ternary_pagerank::*;

// 5-person trust network
let mut g = TernaryGraph::new(5);

// Alice trusts Bob and Carol
g.add_edge(0, 1, 1);  // Alice → Bob: trust
g.add_edge(0, 2, 1);  // Alice → Carol: trust

// Bob trusts Alice, distrusts Dave
g.add_edge(1, 0, 1);  // Bob → Alice: trust
g.add_edge(1, 3, -1); // Bob → Dave: distrust

// Carol trusts everyone except Eve
g.add_edge(2, 0, 1);
g.add_edge(2, 1, 1);
g.add_edge(2, 3, 1);
g.add_edge(2, 4, -1); // Carol → Eve: distrust

// Dave trusts Eve (contrarian)
g.add_edge(3, 4, 1);
g.add_edge(3, 1, -1); // Dave → Bob: distrust (mutual)

// Eve trusts Dave back
g.add_edge(4, 3, 1);
g.add_edge(4, 2, -1); // Eve → Carol: distrust (mutual)

// Communities: who trusts whom?
let communities = g.communities();
println!("Trust communities: {:?}", communities);
// Expected: {Alice, Bob, Carol} and {Dave, Eve}
// The mutual distrust edges form boundaries

// Who's most important in each community?
let pr = g.pagerank(30);
for (node, &score) in pr.iter().enumerate() {
    println!("Node {}: importance = {}", node, score);
}
```

## Design Decisions

**`#![no_std]`** — Network analysis shouldn't require an OS. This crate runs in embedded systems, WASM, and kernel contexts.

**Adjacency matrix** — O(n²) space, but O(1) edge lookup. For the graph sizes this crate targets (tens to low hundreds of nodes), the matrix is cache-friendly and fast. For large graphs, you'd want a sparse representation.

**Ternary clamping** — All intermediate scores are clamped to {-1, 0, +1}. This prevents score explosion and keeps the semantics clear. The tradeoff is some information loss (you can't distinguish between "slightly important" and "very important"), but the three-way classification is the point.

**Betweenness simplification** — Betweenness counts length-2 paths rather than all shortest paths. Full betweenness requires BFS from every node (O(n × (V + E))), which is expensive. The length-2 approximation captures the most important cases: nodes that broker connections between other nodes.

## Ecosystem Connections

- **`ternary-graph`** — Lower-level graph algorithms for ternary-weighted edges
- **`ternary-network`** — Network science (degree distributions, clustering) for ternary graphs
- **`ternary-topology`** — Topological analysis (connected components, Betti numbers)
- **`ternary-ecology`** — Population dynamics on ternary food webs (uses similar graph structure)
- **`ternary-symmetry`** — Group-theoretic symmetry operations on graph automorphisms

## Open Questions

- **Weighted PageRank**: The current implementation treats all +1 edges equally. Should the PageRank propagation weight edges by the source's score? (This is standard PageRank, but the ternary clamping makes it non-trivial.)
- **Convergence guarantees**: Does ternary PageRank always converge? The finite state space ({-1,0,+1}^n) guarantees it doesn't diverge, but it might oscillate.
- **Signed spectral theory**: The eigenvector centrality uses unsigned adjacency for the power iteration. There's a richer theory for signed graphs (using the signed Laplacian) that could extract more information.
- **Scalability**: The adjacency matrix representation limits practical graph sizes. A CSR or edge-list backend would enable larger networks.

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | ~266 |
| Tests | 11 |
| Public API | 11 items |
| Dependencies | 0 (no_std) |

## License

Apache-2.0
