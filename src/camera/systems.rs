use bevy::prelude::*;
use geo_types::{coord, Coord};

use crate::event::{self, ZoomEvent};

use super::{Offset, Scale};

pub fn system_set() -> bevy::ecs::schedule::SystemSet {
    bevy::ecs::schedule::SystemSet::new()
        .with_system(center_camera)
        .with_system(handle_meshes_spawned_event)
        .with_system(zoom)
}

pub fn startup_system_set() -> bevy::ecs::schedule::SystemSet {
    bevy::ecs::schedule::SystemSet::new().with_system(init_camera)
}

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn center_camera(
    windows: Res<Windows>,
    mut query: bevy::prelude::Query<
        &mut bevy::transform::components::Transform,
        bevy::ecs::query::With<bevy::render::camera::Camera>,
    >,
) {
    let window = windows.get_primary().unwrap();
    let center = geo_types::coord! {
        x: ((window.width() / 2.) - (window.width() / 2.)) as f64,
        y: ((window.height() / 2.) - (window.height() / 2.)) as f64,
    };
    let win_width = window.width();
    let win_height = window.height();
    let mut transform = query.single_mut();
    let bevy_size = super::get_bevy_size(win_width, win_height);
    let scale = super::determine_scale(win_width, win_height, bevy_size);
    let mut offset = Offset::from_coord(center);

    offset.pan_x(
        ((window.physical_width() - window.physical_height()) / 2) as f32,
        Scale(scale),
    );
    offset.pan_y(
        ((window.physical_width() - window.physical_height()) / 2) as f32,
        Scale(scale),
    );

    super::set_camera_transform(&mut transform, offset, Scale(scale));
}

fn handle_meshes_spawned_event(
    mut meshes_spawned_event_reader: bevy::ecs::event::EventReader<event::MeshSpawnedEvent>,
    mut zoom_event: bevy::ecs::event::EventWriter<event::ZoomEvent>,
    mut has_moved: bevy::ecs::system::Local<bool>,
) {
    for event in meshes_spawned_event_reader.iter() {
        if !(*has_moved) {
            match &event.0 {
                event::SpawnedBundle::Points(points) => {
                    for geo::Point(coord) in points.iter() {
                        println!("Handle meshes spawned event point");
                        zoom_event.send(ZoomEvent::new(0.5, *coord));
                    }
                }
                event::SpawnedBundle::Mesh(mesh) => {
                    println!("Handle meshes spawned event point");

                    for (i, id) in mesh.attributes().into_iter().enumerate() {
                        if i == 0 as usize {
                            let vert_attr = id.1.as_float3().unwrap();
                            for v in vert_attr {
                                zoom_event.send(ZoomEvent::new(
                                    0.5,
                                    Coord {
                                        x: v[0] as f64,
                                        y: v[1] as f64,
                                    },
                                ));
                            }
                        }
                    }
                }
            }

            *has_moved = true;
        }
    }
}

fn zoom(
    mut zoom_camera_event_reader: bevy::ecs::event::EventReader<event::ZoomEvent>,
    mut query: Query<
        &mut bevy::transform::components::Transform,
        bevy::ecs::query::With<bevy::render::camera::Camera>,
    >,
) {
    if zoom_camera_event_reader.is_empty() {
        return;
    }
    println!("Inside zoom event handler");
    let mut transform = query.single_mut();
    let mut offset = Offset::from_transform(&transform);
    let mut mouse_offset = offset.clone();
    let before_scale = Scale::from_transform(&transform);
    let mut camera_scale = before_scale.clone();
    let mut set = false;

    for event in zoom_camera_event_reader.iter() {
        if !set {
            set = true;
            mouse_offset = Offset::from_coord(event.coord);
        }
        camera_scale.zoom(event.amount);
    }

    if camera_scale.0.is_normal() {
        let xd = mouse_offset.x - offset.x;
        let yd = mouse_offset.y - offset.y;

        offset.x -= xd * (1.0 - before_scale.0 / camera_scale.0);
        offset.y -= yd * (1.0 - before_scale.0 / camera_scale.0);

        if offset.x.is_finite() && offset.y.is_finite() {
            super::set_camera_transform(&mut transform, offset, camera_scale);
        }
    }
}
