# ternary-pagerank

PageRank and centrality measures on ternary-weighted graphs

## Why This Matters

# ternary-pagerank
PageRank and centrality measures on ternary-weighted graphs.
Importance propagation, authority scoring, and community detection.

## The Five-Layer Stack

This crate is part of the **Oxide Stack** — a distributed GPU runtime built on five layers:

```
┌─────────────────┐
│  cudaclaw        │  Persistent GPU kernels, warp consensus, SmartCRDT
├─────────────────┤
│  cuda-oxide      │  Flux → MIR → Pliron → NVVM → PTX compiler
├─────────────────┤
│  flux-core       │  Bytecode VM + A2A agent protocol
├─────────────────┤
│  pincher         │  "Vector DB as runtime, LLM as compiler"
├─────────────────┤
│  open-parallel   │  Async runtime (tokio fork)
└─────────────────┘
```

The key insight: **ternary values {-1, 0, +1} map directly to GPU compute**. They pack 16× denser than FP32, enable XNOR+popcount matmul, and conservation laws become compile-time checks.

## Design

Every value in this crate follows **ternary algebra** (Z₃):

| Value | Meaning | GPU Analog |
|-------|---------|------------|
| +1 | Positive / Active / Healthy | Warp vote yes |
| 0 | Neutral / Pending / Balanced | Warp vote abstain |
| -1 | Negative / Failed / Overloaded | Warp vote no |

This isn't arbitrary — ternary is the natural encoding for:
1. **BitNet b1.58** (Microsoft) — ternary LLMs at 60% less power
2. **GPU warp voting** — hardware ballot returns ternary consensus
3. **Conservation laws** — {-1, 0, +1} preserves quantity

## Key Types

```rust
pub struct TernaryGraph
pub fn new
pub fn add_edge
pub fn out_degree
pub fn in_degree
pub fn pagerank_step
pub fn pagerank
pub fn betweenness
pub fn eigenvector_step
pub fn eigenvector_centrality
pub fn communities
```

## Usage

```toml
[dependencies]
ternary-pagerank = "0.1.0"
```

```rust
use ternary_pagerank::*;
// See src/lib.rs tests for complete working examples
```

## Testing

```bash
git clone https://github.com/SuperInstance/ternary-pagerank.git
cd ternary-pagerank
cargo test    # 11 tests
```

## Stats

| Metric | Value |
|--------|-------|
| Tests | 11 |
| Lines of Rust | 266 |
| Public API | 11 items |

## License

Apache-2.0
