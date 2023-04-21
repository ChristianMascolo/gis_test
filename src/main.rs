use ::bevy::{
    prelude::{App, Color, PluginGroup},
    window::{WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy::{
    prelude::{
        AssetServer, Assets, Bundle, Camera2d, Camera2dBundle, Commands, Component,
        GlobalTransform, Mesh, OrthographicProjection, Query, Res, ResMut, Transform, Vec3,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
};

use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use my_gis::{read_geojson, read_geojson_feature_collection};

mod my_gis;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "rgis".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_startup_system(setup)
        //.add_system(zoom_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/point_and_line.geojson".to_owned()));
    let mut i = 0;

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();
        let mesh_iter = build_bevy_meshes(&geom, Color::GREEN, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();

        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::PreparedMesh::Point(points) => {
                    for geo::Point(coord) in points.iter() {
                        let color = Color::RED;
                        let transform = Transform::from_xyz(
                            coord.x as f32 + (i as f32) * (i as f32),
                            coord.y as f32 + (i as f32) * (i as f32),
                            (i as f32) * (i as f32),
                        );

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

                        i += 1;
                    }
                }
                geo_bevy::PreparedMesh::Polygon { mesh, color } => {
                    println!("Inside Polygon, topology:{:?}", mesh.primitive_topology());
                    println!("Mesh indices: {:?}", mesh.indices());

                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh)),
                        transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                        ..Default::default()
                    });
                }
                geo_bevy::PreparedMesh::LineString { mesh, color } => {
                    println!(
                        "Inside LineString, topology: {:?}",
                        mesh.primitive_topology()
                    );

                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh)),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                }
            }
        }
    }

    commands.spawn(Camera2dBundle {
        camera_2d: bevy::prelude::Camera2d {
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn zoom_camera(mut query: Query<(&Camera2d, &mut OrthographicProjection)>) {
    for (_, mut projection) in query.iter_mut() {
        projection.scale = 2.0;
    }
}
