use serde::{Deserialize, Serialize};
use crate::reconcile::{reconcile, ReconciliationStrategy};
use crate::room_graph::RoomGraph;
use crate::sheaf_state::SheafSections;

/// Convergence analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceAnalysis {
    pub converged: bool,
    pub rounds_to_converge: usize,
    pub convergence_rate: f64,
    pub spectral_gap: f64,
}

/// Analyze convergence of gossip on a room graph.
/// The spectral gap (smallest non-zero eigenvalue of Laplacian) predicts convergence speed.
pub fn analyze_convergence(
    sections: &SheafSections,
    graph: &RoomGraph,
    strategy: &ReconciliationStrategy,
    max_rounds: usize,
    threshold: f64,
) -> ConvergenceAnalysis {
    let spectral_gap = graph.spectral_gap();

    let mut sections_clone = sections.clone();
    let result = reconcile(&mut sections_clone, graph, strategy, max_rounds, threshold);

    // Estimate convergence rate from history
    let convergence_rate = if result.history.len() >= 2 {
        let first = result.history.first().unwrap();
        let last = result.history.last().unwrap();
        if first.h1_before > 1e-15 && result.rounds > 0 {
            // rate ≈ 1 - (h1_final / h1_initial)^(1/rounds)
            let ratio = (last.h1_after / first.h1_before).max(0.0).min(1.0);
            1.0 - ratio.powf(1.0 / result.rounds as f64)
        } else {
            1.0
        }
    } else {
        1.0
    };

    ConvergenceAnalysis {
        converged: result.converged,
        rounds_to_converge: result.rounds,
        convergence_rate,
        spectral_gap,
    }
}

/// Verify that H¹ → 0 for a connected graph given sufficient rounds.
/// Returns the number of rounds needed.
pub fn rounds_to_convergence(
    sections: &SheafSections,
    graph: &RoomGraph,
    strategy: &ReconciliationStrategy,
    threshold: f64,
    max_rounds: usize,
) -> Option<usize> {
    let mut sections_clone = sections.clone();
    let result = reconcile(&mut sections_clone, graph, strategy, max_rounds, threshold);
    if result.converged {
        Some(result.rounds)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sheaf_state::RoomSection;

    #[test]
    fn connected_graph_converges() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![0.0]),
            RoomSection::new("B", vec![5.0]),
            RoomSection::new("C", vec![10.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
            ],
        );
        let analysis = analyze_convergence(
            &sections,
            &graph,
            &ReconciliationStrategy::Averaging { weight: 0.3 },
            500,
            1e-6,
        );
        assert!(analysis.converged);
        assert!(analysis.rounds_to_converge > 0);
    }

    #[test]
    fn complete_graph_faster_than_chain() {
        let data = vec![
            RoomSection::new("A", vec![0.0]),
            RoomSection::new("B", vec![5.0]),
            RoomSection::new("C", vec![10.0]),
        ];
        let sections = SheafSections::new(data.clone());

        let chain = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
            ],
        );
        let complete = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
                ("A".into(), "C".into(), 1.0),
            ],
        );

        let sections2 = SheafSections::new(data);
        let strat = ReconciliationStrategy::Averaging { weight: 0.3 };
        let r1 = rounds_to_convergence(&sections, &chain, &strat, 1e-6, 500);
        let r2 = rounds_to_convergence(&sections2, &complete, &strat, 1e-6, 500);

        // Complete graph should converge in fewer or equal rounds
        if let (Some(rounds_chain), Some(rounds_complete)) = (r1, r2) {
            assert!(rounds_complete <= rounds_chain);
        }
    }

    #[test]
    fn spectral_gap_positive_for_connected() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
            ],
        );
        assert!(graph.spectral_gap() > 0.0);
    }

    #[test]
    fn convergence_rate_positive() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![0.0]),
            RoomSection::new("B", vec![4.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let analysis = analyze_convergence(
            &sections,
            &graph,
            &ReconciliationStrategy::Averaging { weight: 0.5 },
            200,
            1e-6,
        );
        assert!(analysis.convergence_rate > 0.0);
    }

    #[test]
    fn h1_goes_to_zero_bounded_rounds() {
        let sections = SheafSections::new(vec![
            RoomSection::new("A", vec![1.0]),
            RoomSection::new("B", vec![3.0]),
        ]);
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let result = rounds_to_convergence(
            &sections,
            &graph,
            &ReconciliationStrategy::Averaging { weight: 0.5 },
            1e-10,
            100,
        );
        assert!(result.is_some());
        assert!(result.unwrap() <= 100);
    }
}
