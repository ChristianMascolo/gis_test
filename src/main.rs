#![allow(deprecated)]

mod gis_camera;
mod gis_event;
mod gis_id;
mod gis_layers;

use ::bevy::{
    prelude::{App, Color},
    window::{WindowDescriptor, WindowPlugin},
};

use bevy::app::CoreStage;
use bevy::ecs::entity::Entities;
use bevy::prelude::{GlobalTransform, IntoSystemDescriptor, Query, With};
use bevy::sprite::{Material2d, Mesh2dHandle};
use bevy::transform::TransformSystem;
use bevy::{
    prelude::{
        AssetServer, Assets, ClearColor, Commands, EventWriter, Mesh, Res, ResMut, SpatialBundle,
        Vec3,
    },
    render::view::RenderLayers,
    sprite::{Anchor, ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
    window::Windows,
};
use bevy_asset::Handle;

use crate::gis_camera::MyCameraPlugin;
use crate::gis_event::*;
use geo::Rect;
use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use geo_types::Coord;
use gis_test::{read_geojson, read_geojson_feature_collection};
use proj::Transform;
use proj::{Area, Proj};

fn main() {
    let mut app = App::new();

    // resources
    app.insert_resource(ClearColor(Color::rgb(255., 255., 255.)));

    // plugins
    app.add_plugins(bevy::MinimalPlugins);
    app.add_plugin(bevy::asset::AssetPlugin::default());
    app.add_plugin(WindowPlugin {
        window: WindowDescriptor {
            width: 1100.,
            height: 900.,
            title: "gis_test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    });
    app.add_plugin(bevy::winit::WinitPlugin::default());
    app.add_plugin(bevy::render::RenderPlugin::default());
    app.add_plugin(bevy::render::texture::ImagePlugin::default());
    app.add_plugin(bevy::log::LogPlugin::default());
    app.add_plugin(bevy::input::InputPlugin::default());
    app.add_plugin(bevy::core_pipeline::CorePipelinePlugin::default());
    app.add_plugin(bevy::transform::TransformPlugin::default());
    app.add_plugin(bevy::sprite::SpritePlugin::default());
    // app.add_plugin(MyCameraPlugin);

    // events
    // app.add_event::<MeshSpawnedEvent>();
    // app.add_event::<ZoomEvent>();

    // systems
    app.add_startup_system(building_meshes);
    app.add_system_to_stage(
        CoreStage::PostUpdate,
        update_global_transform.after(TransformSystem::TransformPropagate),
    );
    // app.add_system(handle_global_transform);

    // run
    app.run();
}

fn building_meshes(
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    // mut spawned_mesh_event: EventWriter<MeshSpawnedEvent>,
    mut commands: Commands,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(bevy::prelude::Camera2dBundle {
        transform: bevy::prelude::Transform::from_xyz(0.0, 0.0, 999.9),
        ..Default::default()
    });

    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/only_polygon.geojson".to_owned()));

    //window instances
    let win_primary = windows.primary();
    let win_height = win_primary.height();
    let win_width = win_primary.width();
    let win_left = -win_width / 2.0;
    let win_right = win_width / 2.0;
    let win_bottom = -win_height / 2.0;
    let win_top = win_height / 2.0;

    //proj instances
    let from = "EPSG:4326";
    let to = "EPSG:3875";

    /* Calcola il rettangolo che contiene tutte le mesh */
    let mut mesh_rect = Rect::new(
        Coord {
            x: std::f64::MAX,
            y: std::f64::MAX,
        },
        Coord {
            x: std::f64::MIN,
            y: std::f64::MIN,
        },
    );

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();

        // convert geom to proj format
        let area = Area::new(
            win_left as f64,
            win_bottom as f64,
            win_right as f64,
            win_top as f64,
        );
        let proj = Proj::new_known_crs(&from, &to, None).unwrap();
        let result = projection_geometry(geom.clone().into(), proj);

        let mesh_iter = build_bevy_meshes(&geom, Color::RED, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();

        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::PreparedMesh::Point(points) => {
                    for geo::Point(coord) in points.iter() {
                        // spawned_mesh_event
                        //     .send(MeshSpawnedEvent(SpawnedBundle::Points(points.clone())));
                        println!("Coord before transformation {:?}", coord);

                        let color = Color::RED;
                        let mut transform =
                            bevy::prelude::Transform::from_xyz(coord.x as f32, coord.y as f32, 0.);
                        transform.translation = (coord.x as f32, coord.y as f32, 0.).into();

                        let bundle = SpriteBundle {
                            sprite: Sprite {
                                color,
                                ..Default::default()
                            },
                            texture: asset_server.load("circle.png"),
                            transform,
                            ..Default::default()
                        };

                        commands.spawn(bundle);
                    }
                }
                geo_bevy::PreparedMesh::Polygon { mesh, color } => {
                    // spawned_mesh_event.send(MeshSpawnedEvent(SpawnedBundle::Mesh(mesh.clone())));
                    println!("Inside Polygon, topology:{:?}", mesh.primitive_topology());
                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: bevy::prelude::Transform::from_xyz(0., 0., 0.),
                        // .with_scale(Vec3::splat(128.)),
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                }
                geo_bevy::PreparedMesh::LineString { mesh, color } => {
                    // spawned_mesh_event.send(MeshSpawnedEvent(SpawnedBundle::Mesh(mesh.clone())));
                    // let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(vert_attr)) =
                    // mesh.attribute(Mesh::ATTRIBUTE_POSITION);
                    println!("Inside Polygon, topology:{:?}", mesh.primitive_topology());
                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh)),
                        transform: bevy::prelude::Transform::from_xyz(0., 0., 0.),
                        // .with_scale(Vec3::splat(128.)),
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                }
            }
        }
    }
}

fn projection_geometry(geom: geo_types::Geometry, proj: Proj) -> geo_types::Geometry {
    match geom {
        geo_types::Geometry::Point(point) => {
            let new_point = proj.convert(point).unwrap();
            geo_types::Geometry::Point(new_point)
        }
        geo_types::Geometry::Polygon(polygon) => {
            let proj_poly = polygon.transformed(&proj).unwrap();
            geo_types::Geometry::Polygon(proj_poly)
        }
        geo_types::Geometry::LineString(line) => {
            let new_line = line.transformed(&proj).unwrap();
            geo_types::Geometry::LineString(new_line)
        }
        _ => unimplemented!(),
    }
}

type CameraGlobalTransformQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    &'a bevy::transform::components::GlobalTransform,
    (
        bevy::ecs::query::With<bevy::render::camera::Camera>,
        bevy::ecs::query::Changed<bevy::transform::components::GlobalTransform>,
    ),
>;

fn handle_global_transform(
    query: CameraGlobalTransformQuery,
    mut sprite_bundle_query: Query<&mut Sprite>,
) {
    if let Ok(camera_global_transform) = query.get_single() {
        let (scale, _, _) = camera_global_transform.to_scale_rotation_translation();

        for mut sprite in &mut sprite_bundle_query {
            sprite.custom_size = Some(scale.truncate() * 5.);
        }
    }
}

fn update_global_transform(
    query: CameraGlobalTransformQuery,
    mesh: Query<&MaterialMesh2dBundle<Handle<ColorMaterial>>>,
) {
    if let Ok(global_transform) = query.get_single() {
        let (scale, _, _) = global_transform.to_scale_rotation_translation();
        println!("Scale -> {:?}", scale);
        println!("Scale truncate -> {:?}", scale.truncate());
    }

    // let gxf = query.get_single().unwrap();
    // println!("Entity at: {:?}", gxf.translation());
}
