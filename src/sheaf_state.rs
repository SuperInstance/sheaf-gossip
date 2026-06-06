use serde::{Deserialize, Serialize};

/// Local sheaf section for a room: stores the room's data vector and restriction maps
/// to neighboring rooms. Consistency = gluing condition satisfaction (restriction maps
/// must agree on overlaps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSection {
    pub room_id: String,
    pub data: Vec<f64>,
    /// (neighbor_room_id, restricted_data) — the data this room *expects* on the overlap.
    pub restriction_maps: Vec<(String, Vec<f64>)>,
}

impl RoomSection {
    pub fn new(room_id: impl Into<String>, data: Vec<f64>) -> Self {
        Self {
            room_id: room_id.into(),
            data,
            restriction_maps: Vec::new(),
        }
    }

    pub fn with_restriction(mut self, neighbor: impl Into<String>, restricted: Vec<f64>) -> Self {
        self.restriction_maps.push((neighbor.into(), restricted));
        self
    }

    /// Dimension of the data vector.
    pub fn dimension(&self) -> usize {
        self.data.len()
    }
}

/// A collection of sheaf sections over all rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheafSections {
    pub sections: Vec<RoomSection>,
}

impl SheafSections {
    pub fn new(sections: Vec<RoomSection>) -> Self {
        Self { sections }
    }

    pub fn empty() -> Self {
        Self { sections: Vec::new() }
    }

    /// Get a section by room_id.
    pub fn get(&self, room_id: &str) -> Option<&RoomSection> {
        self.sections.iter().find(|s| s.room_id == room_id)
    }

    /// Get a mutable section by room_id.
    pub fn get_mut(&mut self, room_id: &str) -> Option<&mut RoomSection> {
        self.sections.iter_mut().find(|s| s.room_id == room_id)
    }

    /// List all room IDs.
    pub fn room_ids(&self) -> Vec<String> {
        self.sections.iter().map(|s| s.room_id.clone()).collect()
    }
}
