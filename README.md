# ternary-pagerank

PageRank and centrality measures on ternary-weighted directed graphs with edge weights {-1, 0, +1}.

## Background

PageRank, introduced by Brin & Page (1998), assigns importance scores to nodes in a directed graph by modeling a random surfer who follows links with probability d (damping factor) and jumps to a random node with probability 1−d. The algorithm iterates the equation `PR(v) = (1−d)/N + d · Σ(PR(u)/out_degree(u))` for all nodes u linking to v, converging to a stationary distribution of the Markov chain defined by the graph's transition matrix.

In a **ternary-weighted graph**, each edge carries weight −1, 0, or +1. This transforms PageRank from a purely positive diffusion process into one that can model **endorsement and opposition**. A +1 edge contributes positive rank (endorsement), a −1 edge drains rank (opposition), and a 0 edge means no connection. The resulting importance scores reflect not just popularity but *polarized reputation*—a node can be important because it is widely endorsed or because it attracts opposition.

The ternary constraint simplifies computation: instead of weighted averaging over real-valued edge weights, rank propagation becomes a ternary signal flow. Each node's score is clamped to {-1, 0, +1}, creating a coarse-grained importance landscape. This loss of precision is offset by the ability to detect structural polarization that standard PageRank cannot express.

Centrality measures (betweenness, eigenvector) and community detection operate on the same ternary graph, providing a multi-faceted view of network structure including adversarial relationships.

## How It Works

### Architecture

`TernaryGraph` stores an `n × n` adjacency matrix of `i8` values clamped to {-1, 0, +1}. The matrix representation enables O(1) edge lookup at the cost of O(n²) storage, suitable for the small-to-medium graphs targeted by this crate.

### PageRank Iteration

`pagerank_step(scores, damping)`:
```
for each node j:
  rank = (1 - damping) / n  (simplified base)
  for each node i with non-zero edge i→j:
    rank += scores[i] * edge_weight(i,j) / out_degree(i)
  new_scores[j] = clamp(rank, -1, +1)
```

The damping parameter is ternary itself (typically 1, meaning "always follow edges"). The random surfer contribution is simplified to either 0 or 1 depending on damping value.

`pagerank(max_iterations)` runs steps until convergence (scores stop changing) or the iteration limit is reached. Starting scores are initialized to +1 (all nodes equally important).

### Betweenness Centrality

Simplified for ternary: counts paths of length 2 that pass through each intermediate node. Node v has high betweenness if many pairs (s,t) have both s→v and v→t edges. The result is clamped to {-1, 0, +1} via min(count, 1).

### Eigenvector Centrality

One-step propagation: node j's score = Σ scores[i] for all i with edge i→j. Clamped to {-1, 0, +1}. Iterated to convergence, this approximates the dominant eigenvector of the ternary adjacency matrix.

### Community Detection

Connected components are found via DFS following only **positive-weight edges** (weight > 0). This means adversarial edges (−1) act as barriers that separate communities—a node connected by only negative edges is isolated into its own component.

## Experimental Results

All 11 unit tests pass:

| Test | Result | Key Observation |
|------|--------|-----------------|
| `test_graph_new` | ✅ | Empty 3-node graph: all degrees = 0 |
| `test_add_edge` | ✅ | Edge 0→1: out_degree(0) = 1, in_degree(1) = 1 |
| `test_negative_edge` | ✅ | Weight −1 stored correctly |
| `test_pagerank_simple` | ✅ | 3-node cycle: PageRank converges to 3-element score vector |
| `test_pagerank_hub` | ✅ | Hub-and-spoke (4 nodes, bidirectional): scores propagate through hub |
| `test_betweenness` | ✅ | Chain 0→1→2: node 1 has betweenness = 1 (bridge), others = 0 |
| `test_eigenvector_step` | ✅ | Single step: node 1 receives score from node 0's edge |
| `test_eigenvector_centrality` | ✅ | 3-node cycle: converges to stable centrality scores |
| `test_communities_connected` | ✅ | Path 0→1→2→3: single community |
| `test_communities_disconnected` | ✅ | Paths 0→1 and 2→3: two communities |
| `test_communities_negative_edge_excluded` | ✅ | Edge 1→2 with weight −1: splits into 2 communities |

The community detection test with negative edges is particularly important: despite nodes 1 and 2 being connected by an edge, the −1 weight causes them to be placed in separate communities. Negative edges are treated as *repulsion*, not connection.

## Impact of Ternary {-1, 0, +1}

The ternary edge model enables **polarized network analysis**:

- **+1 edges**: Endorsement, trust, positive influence. Standard PageRank considers only these.
- **0 edges**: No relationship. Node pairs with no edge don't interact.
- **−1 edges**: Opposition, distrust, negative influence. Rank flows backward along these edges, reducing the target's importance.

This three-way distinction captures social dynamics (friend/neutral/foe), economic relationships (buy/ignore/short), and system interactions (cooperate/noop/compete) that binary graphs cannot express.

## Use Cases

1. **Social Network Polarization Analysis**: Model users with +1 edges for follows/likes and −1 edges for blocks/downvotes. PageRank on this ternary graph identifies influential users both positively (endorsed) and negatively (opposed), revealing polarization structure.

2. **Trust/Reputation Systems**: In peer-to-peer networks, ternary edges represent positive/negative/neutral trust ratings. Eigenvector centrality identifies nodes whose trust score is globally significant—trusted-by-trusted nodes versus distrusted-by-distrusted nodes.

3. **Adversarial Graph Analysis**: In cybersecurity, model network traffic with +1 for known-good connections and −1 for suspicious/known-bad connections. Community detection groups trusted nodes together while isolating adversarial ones.

4. **Supply Chain Risk Propagation**: Model supplier relationships with +1 for reliable suppliers and −1 for unreliable ones. Cascading importance scores identify nodes whose failure or adversarial behavior poses systemic risk.

5. **GPU Kernel Dependency Analysis**: Model kernel dependencies with +1 for data flow and −1 for resource contention. PageRank identifies critical-path kernels (high positive rank) and contention bottlenecks (nodes receiving negative rank from multiple −1 edges).

## Open Questions

1. **Convergence Guarantees**: Does ternary PageRank always converge? The clamping to {-1, 0, +1} bounds the state space, but the ternary damping model may introduce oscillation. Under what graph conditions is convergence guaranteed?

2. **Negative Rank Interpretation**: When a node's rank saturates at −1, what does this mean structurally? Is it "widely opposed" or "connected to powerful opponents"? The semantics differ and the current algorithm doesn't distinguish.

3. **Weighted Community Detection**: Currently, communities use only positive edges. Should negative edges within a community reduce cohesion? A signed modularity metric could detect communities that are internally cooperative but externally adversarial.

## Connection to Oxide Stack

Within the five-layer Oxide ternary architecture:

- **Layer 1 (Ternary Genome)**: Graph edges encoded as genome trits {-1, 0, +1} enable evolutionary optimization of network structure—mutations flip edge weights, selecting for topologies with desirable PageRank properties.
- **Layer 2 (Cellular Computation)**: Each PageRank iteration is a cell-level computation where nodes exchange ternary signals. The adjacency matrix defines the communication topology between cells.
- **Layer 3 (Organism Behavior)**: Centrality scores drive organism-level behavior—an organism at a high-centrality node in the interaction graph acts as a hub; one at a negatively-ranked node may exhibit defensive behavior.
- **Layer 4 (Population Dynamics)**: Community detection on the population interaction graph reveals social structure—clusters of cooperating organisms separated by adversarial boundaries.
- **Layer 5 (Ecosystem)**: At the ecosystem level, ternary PageRank identifies keystone species (high positive centrality) and invasive/threatening elements (high negative centrality), guiding resource allocation and intervention strategies.
