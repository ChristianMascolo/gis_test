use ::bevy::{
    prelude::{App, Color, PluginGroup},
    window::{WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};

use bevy::{
    prelude::{
        AssetServer, Assets, Camera2dBundle, Commands, Mesh, Res, ResMut, Transform, Vec2, Query, OrthographicProjection, With, Camera,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle}, time::Time,
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
        //.add_system(setup)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    // commands.spawn(SpriteBundle {
    //     sprite: Sprite {
    //         custom_size: Some(Vec2 { x: 50., y: 50. }),
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });
}

fn zoom_camera(mut query: Query<&mut OrthographicProjection, With<Camera>>, time: Res<Time>){
    for mut projection in query.iter_mut() {
        let mut log_scale = projection.scale.ln();
        log_scale -= 0.1 * time.delta_seconds();
        projection.scale = log_scale.exp();

        println!("Current zoom scale: {}", projection.scale);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/only_polygon.geojson".to_owned()));
    let mut i: f32 = 0.;

    commands.spawn(Camera2dBundle::default());

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();
        let mesh_iter = build_bevy_meshes(&geom, Color::GREEN, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();

        // println!("geom: {:?}", geom);
        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::PreparedMesh::Point(points) => {
                    for geo::Point(coord) in points.iter() {
                        let color = Color::RED;
                        let mut offset = Vec2::new(coord.x as f32, coord.y as f32);
                        offset *= 2.0; // Modifica la costante dell'offset come preferisci
                        let mut transform =
                            Transform::from_xyz(coord.x as f32, coord.y as f32, i * i);
                        transform.translation += offset.extend(0.0); // Aggiunge l'offset alla posizione di traduzione

                        println!("{:?}", transform.translation);

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

                        i += 1.;
                    }
                }
                geo_bevy::PreparedMesh::Polygon { mesh, color } => {
                    println!("Inside Polygon, topology:{:?}", mesh.primitive_topology());
                    println!("Mesh indices: {:?}",mesh.indices());
                    let material = materials.add(color.into());
                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh)),
                        transform: Transform::from_xyz(0., 0., 0.),
                        visibility: bevy::render::view::Visibility { is_visible: true },
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
                        transform: Transform::from_xyz(0., 0., 1.),
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                }
            }
        }
    }
}
