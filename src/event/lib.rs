use bevy::prelude::Mesh;
use geo_bevy::PreparedMesh;
use geo_types::Point;

pub enum SpawnedBundle{
    Points(Vec<Point>),
    Mesh(Mesh)
}

pub struct MeshSpawnedEvent(pub SpawnedBundle);

pub struct DespawnedMesh(pub PreparedMesh);

pub struct ZoomEvent {
    /// * `amount ∈ (1, ∞)` → zoom in
    /// * `amount ∈ [1]` → no change
    /// * `amount ∈ (0, 1)` → zoom out
    pub amount: f32,
    pub coord: geo::Coord,
}

impl ZoomEvent {
    #[inline]
    pub fn new(amount: f32, coord: geo::Coord) -> Self {
        ZoomEvent {
            // Don't let amount be negative, so add `max`
            amount: (1. + amount / 500.).max(0.),
            coord,
        }
    }
}
