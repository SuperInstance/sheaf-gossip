use serde::{Deserialize, Serialize};
use crate::room_graph::RoomGraph;
use crate::sheaf_state::SheafSections;

/// H¹ obstruction: quantifies the failure of local sections to glue into a global section.
/// magnitude > 0 means the sheaf cohomology H¹ is non-trivial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstruction {
    /// Total magnitude of mismatch (L2 norm of coboundary).
    pub magnitude: f64,
    /// Each edge where restrictions disagree: (room_a, room_b, mismatch_value).
    pub mismatch_edges: Vec<(String, String, f64)>,
    /// Approximate dimension of H¹ (number of linearly independent obstructions).
    pub h1_dimension: usize,
}

/// Compute the H¹ obstruction for the given sections on the given room graph.
/// For each edge (u, v) in the graph, we check whether u's restriction to v
/// matches v's restriction to u (or v's data if no explicit restriction).
/// The mismatch is the L2 norm of the difference.
pub fn compute_obstruction(sections: &SheafSections, graph: &RoomGraph) -> Obstruction {
    let mut mismatch_edges: Vec<(String, String, f64)> = Vec::new();
    let mut total_magnitude: f64 = 0.0;

    for (u, v, _weight) in &graph.edges {
        let mismatch = compute_edge_mismatch(sections, u, v);
        if mismatch > 1e-15 {
            mismatch_edges.push((u.clone(), v.clone(), mismatch));
            total_magnitude += mismatch * mismatch;
        }
    }

    total_magnitude = total_magnitude.sqrt();

    // H¹ dimension ≈ number of independent mismatch edges (simple heuristic).
    let h1_dimension = mismatch_edges.len();

    Obstruction {
        magnitude: total_magnitude,
        mismatch_edges,
        h1_dimension,
    }
}

/// Compute mismatch on a single edge.
/// If room u has a restriction map to v, we compare that restricted data with v's data.
/// If room v has a restriction map to u, we compare that with u's data.
/// We take the average of both directional mismatches.
fn compute_edge_mismatch(sections: &SheafSections, u: &str, v: &str) -> f64 {
    let sec_u = sections.get(u);
    let sec_v = sections.get(v);

    match (sec_u, sec_v) {
        (Some(su), Some(sv)) => {
            // Forward: u restricts to overlap with v
            let forward = su.restriction_maps.iter()
                .find(|(id, _)| id == v)
                .map(|(_, restricted)| {
                    l2_distance(restricted, &sv.data)
                })
                .unwrap_or(0.0);

            // Backward: v restricts to overlap with u
            let backward = sv.restriction_maps.iter()
                .find(|(id, _)| id == u)
                .map(|(_, restricted)| {
                    l2_distance(restricted, &su.data)
                })
                .unwrap_or(0.0);

            // Also check direct data difference as baseline
            let direct = l2_distance(&su.data, &sv.data);

            // If no restriction maps, use direct difference
            if su.restriction_maps.iter().any(|(id, _)| id == v)
                || sv.restriction_maps.iter().any(|(id, _)| id == u)
            {
                (forward + backward) / 2.0
            } else {
                direct
            }
        }
        _ => 0.0,
    }
}

fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

/// Compute the obstruction for the trivial case: sections where data is directly
/// compared across edges (no explicit restriction maps).
pub fn compute_direct_obstruction(sections: &SheafSections, graph: &RoomGraph) -> Obstruction {
    let mut mismatch_edges: Vec<(String, String, f64)> = Vec::new();
    let mut total_sq: f64 = 0.0;

    for (u, v, _weight) in &graph.edges {
        if let (Some(su), Some(sv)) = (sections.get(u), sections.get(v)) {
            let d = l2_distance(&su.data, &sv.data);
            if d > 1e-15 {
                mismatch_edges.push((u.clone(), v.clone(), d));
                total_sq += d * d;
            }
        }
    }

    Obstruction {
        magnitude: total_sq.sqrt(),
        h1_dimension: mismatch_edges.len(),
        mismatch_edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sheaf_state::RoomSection;

    #[test]
    fn single_room_h1_zero() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![1.0, 2.0, 3.0]),
        ]);
        let graph = RoomGraph::new(vec!["A".into()], vec![]);
        let obs = compute_direct_obstruction(&sections, &graph);
        assert!((obs.magnitude - 0.0).abs() < 1e-10);
        assert_eq!(obs.h1_dimension, 0);
    }

    #[test]
    fn two_rooms_same_data_h1_zero() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![1.0, 2.0]),
            RoomSection::new("B", vec![1.0, 2.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let obs = compute_direct_obstruction(&sections, &graph);
        assert!(obs.magnitude < 1e-10);
        assert_eq!(obs.h1_dimension, 0);
    }

    #[test]
    fn two_rooms_different_data_h1_positive() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![1.0, 2.0]),
            RoomSection::new("B", vec![3.0, 4.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let obs = compute_direct_obstruction(&sections, &graph);
        assert!(obs.magnitude > 0.0);
        assert_eq!(obs.h1_dimension, 1);
        // magnitude = sqrt((1-3)^2 + (2-4)^2) = sqrt(8) ≈ 2.828
        assert!((obs.magnitude - 8.0_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn obstruction_with_restriction_maps() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![1.0, 2.0])
                .with_restriction("B", vec![1.5, 2.5]),
            RoomSection::new("B", vec![3.0, 4.0])
                .with_restriction("A", vec![2.5, 3.5]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let obs = compute_obstruction(&sections, &graph);
        assert!(obs.magnitude > 0.0);
    }

    #[test]
    fn three_rooms_chain_obstruction() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![0.0]),
            RoomSection::new("B", vec![1.0]),
            RoomSection::new("C", vec![2.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
            ],
        );
        let obs = compute_direct_obstruction(&sections, &graph);
        assert!(obs.magnitude > 0.0);
        assert_eq!(obs.h1_dimension, 2);
    }

    #[test]
    fn fully_consistent_sections_zero_obstruction() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![5.0])
                .with_restriction("B", vec![5.0]),
            RoomSection::new("B", vec![5.0])
                .with_restriction("A", vec![5.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let obs = compute_obstruction(&sections, &graph);
        assert!(obs.magnitude < 1e-10);
    }
}
