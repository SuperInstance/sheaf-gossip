use serde::{Deserialize, Serialize};

/// Room connectivity graph: which rooms share agents/passages.
/// Edge weights represent connection strength / bandwidth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomGraph {
    pub rooms: Vec<String>,
    /// (room_a, room_b, weight)
    pub edges: Vec<(String, String, f64)>,
}

impl RoomGraph {
    pub fn new(rooms: Vec<String>, edges: Vec<(String, String, f64)>) -> Self {
        Self { rooms, edges }
    }

    /// Build a complete graph with uniform weights.
    pub fn complete(rooms: Vec<String>, weight: f64) -> Self {
        let mut edges = Vec::new();
        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                edges.push((rooms[i].clone(), rooms[j].clone(), weight));
            }
        }
        Self { rooms, edges }
    }

    /// Number of rooms (vertices).
    pub fn num_rooms(&self) -> usize {
        self.rooms.len()
    }

    /// Number of edges.
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    /// Get neighbors of a room.
    pub fn neighbors(&self, room_id: &str) -> Vec<&str> {
        self.edges.iter()
            .filter_map(|(u, v, _)| {
                if u == room_id { Some(v.as_str()) }
                else if v == room_id { Some(u.as_str()) }
                else { None }
            })
            .collect()
    }

    /// Check if the graph is connected (BFS/DFS).
    pub fn is_connected(&self) -> bool {
        if self.rooms.is_empty() {
            return true;
        }

        let mut visited = vec![false; self.rooms.len()];
        let room_index: Vec<&str> = self.rooms.iter().map(|s| s.as_str()).collect();

        let start = 0;
        let mut stack = vec![start];
        visited[start] = true;
        let mut count = 1;

        while let Some(node) = stack.pop() {
            let room_id = self.rooms[node].as_str();
            for neighbor in self.neighbors(room_id) {
                if let Some(idx) = room_index.iter().position(|&r| r == neighbor) {
                    if !visited[idx] {
                        visited[idx] = true;
                        count += 1;
                        stack.push(idx);
                    }
                }
            }
        }

        count == self.rooms.len()
    }

    /// Compute the Laplacian matrix of the graph.
    /// L[i][j] = -weight(i,j) if i != j, L[i][i] = sum of weights of edges incident to i.
    pub fn laplacian(&self) -> Vec<Vec<f64>> {
        let n = self.rooms.len();
        let mut l = vec![vec![0.0; n]; n];

        for (u, v, w) in &self.edges {
            let i = self.rooms.iter().position(|r| r == u).unwrap_or(usize::MAX);
            let j = self.rooms.iter().position(|r| r == v).unwrap_or(usize::MAX);
            if i != usize::MAX && j != usize::MAX {
                l[i][j] -= w;
                l[j][i] -= w;
                l[i][i] += w;
                l[j][j] += w;
            }
        }

        l
    }

    /// Compute eigenvalues of the Laplacian using the power method / QR-like approach.
    /// For small matrices, we use Jacobi eigenvalue algorithm for symmetric matrices.
    pub fn laplacian_eigenvalues(&self) -> Vec<f64> {
        let n = self.rooms.len();
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![0.0];
        }

        let mut a = self.laplacian();

        // Simple QR iteration for symmetric tridiagonal matrices
        // First, reduce to tridiagonal form is complex. Use direct Jacobi instead.
        jacobi_eigenvalues(&mut a, n)
    }

    /// Spectral gap: the smallest non-zero eigenvalue of the Laplacian.
    /// For a connected graph, this is λ₂.
    pub fn spectral_gap(&self) -> f64 {
        let eigenvalues = self.laplacian_eigenvalues();
        let mut sorted = eigenvalues.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // For a graph with k components, the first k eigenvalues are 0.
        // Spectral gap = λ₂ = second smallest eigenvalue.
        // For connected graph, λ₁=0, λ₂>0.
        // For disconnected, λ₂=0.
        if sorted.len() >= 2 {
            sorted[1]
        } else {
            0.0
        }
    }
}

/// Jacobi eigenvalue algorithm for symmetric matrices.
fn jacobi_eigenvalues(a: &mut Vec<Vec<f64>>, n: usize) -> Vec<f64> {
    let max_iterations = 1000 * n;
    let tolerance = 1e-14;

    for _ in 0..max_iterations {
        // Find the largest off-diagonal element
        let mut max_val = 0.0;
        let mut p = 0;
        let mut q = 1;
        for i in 0..n {
            for j in (i + 1)..n {
                if a[i][j].abs() > max_val {
                    max_val = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }

        if max_val < tolerance {
            break;
        }

        // Compute rotation angle
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        let theta = if (app - aqq).abs() < 1e-15 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * (2.0 * apq / (app - aqq)).atan()
        };

        let c = theta.cos();
        let s = theta.sin();

        // Apply Givens rotation
        let mut new_a = a.clone();
        for i in 0..n {
            if i != p && i != q {
                new_a[i][p] = c * a[i][p] + s * a[i][q];
                new_a[p][i] = new_a[i][p];
                new_a[i][q] = -s * a[i][p] + c * a[i][q];
                new_a[q][i] = new_a[i][q];
            }
        }
        new_a[p][p] = c * c * app + 2.0 * s * c * apq + s * s * aqq;
        new_a[q][q] = s * s * app - 2.0 * s * c * apq + c * c * aqq;
        new_a[p][q] = 0.0;
        new_a[q][p] = 0.0;

        *a = new_a;
    }

    // Extract diagonal as eigenvalues
    (0..n).map(|i| a[i][i]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_connected() {
        let graph = RoomGraph::new(vec![], vec![]);
        assert!(graph.is_connected());
    }

    #[test]
    fn single_room_is_connected() {
        let graph = RoomGraph::new(vec!["A".into()], vec![]);
        assert!(graph.is_connected());
    }

    #[test]
    fn two_rooms_with_edge_connected() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        assert!(graph.is_connected());
    }

    #[test]
    fn two_rooms_no_edge_disconnected() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![],
        );
        assert!(!graph.is_connected());
    }

    #[test]
    fn three_rooms_chain_connected() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("B".into(), "C".into(), 1.0),
            ],
        );
        assert!(graph.is_connected());
    }

    #[test]
    fn three_rooms_disconnected_component() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        assert!(!graph.is_connected());
    }

    #[test]
    fn complete_graph_construction() {
        let graph = RoomGraph::complete(vec!["A".into(), "B".into(), "C".into()], 1.0);
        assert_eq!(graph.num_rooms(), 3);
        assert_eq!(graph.num_edges(), 3);
        assert!(graph.is_connected());
    }

    #[test]
    fn neighbors_correct() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![
                ("A".into(), "B".into(), 1.0),
                ("A".into(), "C".into(), 1.0),
            ],
        );
        let mut nbrs = graph.neighbors("A");
        nbrs.sort();
        assert_eq!(nbrs, vec!["B", "C"]);
        assert_eq!(graph.neighbors("B"), vec!["A"]);
    }

    #[test]
    fn laplacian_single_edge() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 2.0)],
        );
        let l = graph.laplacian();
        assert!((l[0][0] - 2.0).abs() < 1e-10);
        assert!((l[1][1] - 2.0).abs() < 1e-10);
        assert!((l[0][1] - (-2.0)).abs() < 1e-10);
        assert!((l[1][0] - (-2.0)).abs() < 1e-10);
    }

    #[test]
    fn eigenvalues_include_zero() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let eigs = graph.laplacian_eigenvalues();
        assert!(eigs.iter().any(|e| e.abs() < 1e-8));
    }

    #[test]
    fn spectral_gap_connected_positive() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        let gap = graph.spectral_gap();
        assert!(gap > 0.0);
        // For single edge with weight 1, λ₂ = 2.0
        assert!((gap - 2.0).abs() < 0.5);
    }

    #[test]
    fn spectral_gap_disconnected_zero() {
        let graph = RoomGraph::new(
            vec!["A".into(), "B".into(), "C".into()],
            vec![("A".into(), "B".into(), 1.0)],
        );
        // C is disconnected, so λ₂ ≈ 0
        let gap = graph.spectral_gap();
        assert!(gap < 0.1);
    }
}
