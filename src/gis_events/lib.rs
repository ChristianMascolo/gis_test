pub struct CreateLayerEventWriter(pub i32); //la struttura contiene l'id del layer creato

pub struct CenterCameraEvent(pub i32); //contiene l'id del layer di riferimento

pub struct MeshSpawnedEvent(pub i32); //contiene l'id del layer dove viene generata la mesh

#[derive(Debug)]
pub struct PanCameraEvent {
    // X offset for camera position. Positive is right, negative is left.
    pub x: f32,
    // Y offset for camera position. Positive is up, negative is down.
    pub y: f32,
}

impl PanCameraEvent {
    #[inline]
    pub fn up(amount: f32) -> Self {
        PanCameraEvent { x: 0., y: amount }
    }

    #[inline]
    pub fn right(amount: f32) -> Self {
        PanCameraEvent { x: amount, y: 0. }
    }

    #[inline]
    pub fn down(amount: f32) -> Self {
        PanCameraEvent { x: 0., y: -amount }
    }

    #[inline]
    pub fn left(amount: f32) -> Self {
        PanCameraEvent { x: -amount, y: 0. }
    }
}

#[derive(Debug)]
pub struct ZoomCameraEvent {
    /// * `amount ∈ (1, ∞)` → zoom in
    /// * `amount ∈ [1]` → no change
    /// * `amount ∈ (0, 1)` → zoom out
    pub amount: f32,
    pub coord: geo::Coord,
}

impl ZoomCameraEvent {
    #[inline]
    pub fn new(amount: f32, coord: geo::Coord) -> Self {
        ZoomCameraEvent {
            // Don't let amount be negative, so add `max`
            amount: (1. + amount / 500.).max(0.),
            coord,
        }
    }
}

pub struct CreateLayerEventReader{
    pub name: String,
    pub crs: String,
    pub coord: geo::Coord,
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin{
    fn build(&self, app: &mut App){
        app.add_event::<LayerCreatedEvent>()
            .add_event::<CenterCameraEvent>()
            .add_event::<MeshSpawnedEvent>()
            .add_event::<PanCameraEvent>()
            .add_event::<ZoomCameraEvent>()
            .add_event::<NewLayerEvent>()
    }
}