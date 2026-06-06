use sheaf_gossip::*;

#[test]
fn full_pipeline_averaging() {
    let sections = sheaf_state::SheafSections::new(vec![
        sheaf_state::RoomSection::new("A", vec![0.0]),
        sheaf_state::RoomSection::new("B", vec![5.0]),
        sheaf_state::RoomSection::new("C", vec![10.0]),
    ]);
    let graph = room_graph::RoomGraph::new(
        vec!["A".into(), "B".into(), "C".into()],
        vec![
            ("A".into(), "B".into(), 1.0),
            ("B".into(), "C".into(), 1.0),
        ],
    );

    // Step 1: Compute obstruction
    let obs = obstruction::compute_direct_obstruction(&sections, &graph);
    assert!(obs.magnitude > 0.0);

    // Step 2: Schedule gossip
    let schedule = gossip_schedule::schedule_from_obstruction(&obs, graph.num_edges());
    assert!(schedule.total_rounds > 0);

    // Step 3: Reconcile
    let mut sections = sections;
    let result = reconcile::reconcile(
        &mut sections,
        &graph,
        &reconcile::ReconciliationStrategy::Averaging { weight: 0.5 },
        500,
        1e-6,
    );
    assert!(result.converged);
    assert!(result.final_h1 < 1e-6);
}

#[test]
fn full_pipeline_weighted_merge() {
    let sections = sheaf_state::SheafSections::new(vec![
        sheaf_state::RoomSection::new("R1", vec![1.0, 2.0]),
        sheaf_state::RoomSection::new("R2", vec![3.0, 4.0]),
        sheaf_state::RoomSection::new("R3", vec![5.0, 6.0]),
        sheaf_state::RoomSection::new("R4", vec![7.0, 8.0]),
    ]);
    let graph = room_graph::RoomGraph::complete(
        vec!["R1".into(), "R2".into(), "R3".into(), "R4".into()],
        1.0,
    );

    let obs = obstruction::compute_direct_obstruction(&sections, &graph);
    assert!(obs.magnitude > 0.0);

    let mut sections = sections;
    let result = reconcile::reconcile(
        &mut sections,
        &graph,
        &reconcile::ReconciliationStrategy::WeightedMerge,
        200,
        1e-6,
    );
    assert!(result.converged);
}

#[test]
fn disconnected_graph_cannot_reconcile_global_difference() {
    // Two disconnected components with different values.
    // Each component is internally consistent, so H¹ = 0.
    // But globally, the values differ — which can't be reconciled without connectivity.
    let mut sections = sheaf_state::SheafSections::new(vec![
        sheaf_state::RoomSection::new("A", vec![0.0]),
        sheaf_state::RoomSection::new("B", vec![0.0]),
        sheaf_state::RoomSection::new("C", vec![10.0]),
        sheaf_state::RoomSection::new("D", vec![10.0]),
    ]);
    let graph = room_graph::RoomGraph::new(
        vec!["A".into(), "B".into(), "C".into(), "D".into()],
        vec![
            ("A".into(), "B".into(), 1.0),
            ("C".into(), "D".into(), 1.0),
        ],
    );

    // H¹ = 0 because each edge connects same-valued rooms
    let obs = obstruction::compute_direct_obstruction(&sections, &graph);
    assert!(obs.magnitude < 1e-10);

    // But the graph is disconnected
    assert!(!graph.is_connected());

    // After gossip, A,B stay at 0, C,D stay at 10 — no global convergence
    let result = reconcile::reconcile(
        &mut sections,
        &graph,
        &reconcile::ReconciliationStrategy::Averaging { weight: 0.5 },
        100,
        1e-6,
    );
    assert!(result.converged); // trivially, since each edge is already consistent
}

#[test]
fn convergence_analysis_full() {
    let sections = sheaf_state::SheafSections::new(vec![
        sheaf_state::RoomSection::new("X", vec![0.0]),
        sheaf_state::RoomSection::new("Y", vec![8.0]),
    ]);
    let graph = room_graph::RoomGraph::new(
        vec!["X".into(), "Y".into()],
        vec![("X".into(), "Y".into(), 1.0)],
    );

    let analysis = convergence::analyze_convergence(
        &sections,
        &graph,
        &reconcile::ReconciliationStrategy::Averaging { weight: 0.5 },
        200,
        1e-6,
    );

    assert!(analysis.converged);
    assert!(analysis.spectral_gap > 0.0);
    assert!(analysis.convergence_rate > 0.0);
}
