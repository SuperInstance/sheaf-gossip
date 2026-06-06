# sheaf-gossip

**Bridging sheaf cohomology to room gossip protocols.**

When H¹ ≠ 0, rooms know they disagree and need reconciliation messages. Sheaf obstructions drive the gossip schedule.

## Core Idea

In a distributed system of *rooms* (nodes) connected by edges (shared agents/passages), each room maintains a local *sheaf section* — a vector of data. The question: **do these local views glue into a consistent global state?**

The answer lives in sheaf cohomology. When the first cohomology group H¹ is non-trivial, local sections conflict on overlaps — the system is inconsistent. The magnitude of the obstruction quantifies *how badly* they disagree.

This library computes that obstruction and uses it to drive a gossip reconciliation protocol:

1. **Detect** — Compute H¹ obstruction from local sections and the room graph.
2. **Schedule** — Higher obstruction → more aggressive gossip (t-minus campaign topology).
3. **Reconcile** — Averaging, weighted merge, or majority vote to reduce H¹ each round.
4. **Converge** — H¹ → 0 under sufficient connectivity, at a rate governed by the spectral gap of the room graph's Laplacian.

## Modules

| Module | Purpose |
|--------|---------|
| `sheaf_state` | Local sheaf sections per room: data vectors and restriction maps. |
| `obstruction` | Compute H¹ obstruction: quantify mismatches at overlaps. |
| `gossip_schedule` | Schedule reconciliation messages from obstruction magnitude. |
| `reconcile` | Reconciliation protocols: averaging, weighted merge, majority vote. |
| `convergence` | Track convergence; spectral gap predicts speed. |
| `room_graph` | Room connectivity graph; Laplacian eigenvalues. |

## Core Types

```rust
struct RoomSection {
    room_id: String,
    data: Vec<f64>,
    restriction_maps: Vec<(String, Vec<f64>)>,
}

struct Obstruction {
    magnitude: f64,
    mismatch_edges: Vec<(String, String, f64)>,
    h1_dimension: usize,
}

struct GossipRound {
    round: usize,
    messages_sent: usize,
    h1_before: f64,
    h1_after: f64,
}

struct ReconciliationResult {
    converged: bool,
    rounds: usize,
    final_h1: f64,
    history: Vec<GossipRound>,
}

struct RoomGraph {
    rooms: Vec<String>,
    edges: Vec<(String, String, f64)>,
}
```

## Quick Start

```rust
use sheaf_gossip::*;

// Define rooms and their data
let sections = sheaf_state::SheafSections::new(vec![
    sheaf_state::RoomSection::new("kitchen", vec![22.0]),
    sheaf_state::RoomSection::new("living_room", vec![24.0]),
    sheaf_state::RoomSection::new("bedroom", vec![20.0]),
]);

// Define connectivity
let graph = room_graph::RoomGraph::new(
    vec!["kitchen".into(), "living_room".into(), "bedroom".into()],
    vec![
        ("kitchen".into(), "living_room".into(), 1.0),
        ("living_room".into(), "bedroom".into(), 1.0),
    ],
);

// Compute obstruction (H¹)
let obs = obstruction::compute_direct_obstruction(&sections, &graph);
println!("H¹ magnitude: {:.4}", obs.magnitude);

// Schedule gossip
let schedule = gossip_schedule::schedule_from_obstruction(&obs, graph.num_edges());
println!("Rounds needed: {}", schedule.total_rounds);

// Reconcile
let mut sections = sections;
let result = reconcile::reconcile(
    &mut sections,
    &graph,
    &reconcile::ReconciliationStrategy::Averaging { weight: 0.5 },
    500,
    1e-6,
);
println!("Converged: {} in {} rounds (final H¹: {:.6})",
    result.converged, result.rounds, result.final_h1);
```

## Mathematical Background

### Sheaf Cohomology

A **sheaf** assigns data to each open set (room) with restriction maps between them. Given a covering of rooms connected by overlaps (edges), local sections `s_i ∈ F(U_i)` are **consistent** when they agree on overlaps:

```
s_i|_{U_i ∩ U_j} = s_j|_{U_i ∩ U_j}  for all i, j
```

When this fails, the obstruction lives in H¹ — the first Čech cohomology group.

### Gossip Convergence

The reconciliation protocol is a consensus iteration on the room graph. Each round, rooms blend their data with neighbors. The convergence rate is governed by the **spectral gap** (λ₂ of the graph Laplacian):

- **Connected graph**: λ₂ > 0 → consensus reached, H¹ → 0
- **Disconnected graph**: λ₂ = 0 → global consensus impossible
- **Higher spectral gap** → faster convergence

For averaging with weight `w`, the convergence rate is approximately:

```
rate ≈ 1 - (1 - w · λ₂/N)^rounds
```

## Reconciliation Strategies

| Strategy | Best For | Mechanism |
|----------|----------|-----------|
| **Averaging** | Continuous data | Blend with neighbors: `x_new = (1-w)·x + w·x̄_neighbors` |
| **WeightedMerge** | Heterogeneous bandwidth | Weight by edge weights |
| **MajorityVote** | Discrete/categorical | Snap to most common value within tolerance |

## Testing

```bash
cargo test
```

38 tests covering:
- Trivial H¹ = 0 (single room, consistent rooms)
- H¹ > 0 detection (conflicting data)
- Gossip round reduces H¹
- Convergence in bounded rounds for connected graphs
- Spectral gap predicts convergence speed
- Disconnected graphs: no global reconciliation
- Full pipeline: sections → obstruction → schedule → reconcile → converge

## License

MIT
