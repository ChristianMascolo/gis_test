#![allow(deprecated)]

mod gis_camera;
mod gis_event;
mod gis_layer_id;
mod gis_layers;

use ::bevy::{
    prelude::{App, Color},
    window::{WindowDescriptor, WindowPlugin},
};

use bevy::{
    prelude::{
        AssetServer, Assets, ClearColor, Commands, Mesh, Res, ResMut,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
};

// use crate::gis_camera::MyCameraPlugin;
// use crate::gis_event::*;
use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use gis_test::{read_geojson, read_geojson_feature_collection};
use proj::Transform;
use proj::Proj;

fn main() {
    let mut app = App::new();    

    app.add_plugin(WindowPlugin {
        window: WindowDescriptor {
            width: 1100.,
            height: 900.,
            title: "gis_test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    });

    // resources
    app.insert_resource(ClearColor(Color::rgb(255., 255., 255.)));

    // plugins
    app.add_plugins(bevy::MinimalPlugins);
    app.add_plugin(bevy::asset::AssetPlugin::default());
    app.add_plugin(bevy::winit::WinitPlugin::default());
    app.add_plugin(bevy::render::RenderPlugin::default());
    app.add_plugin(bevy::render::texture::ImagePlugin::default());
    app.add_plugin(bevy::log::LogPlugin::default());
    app.add_plugin(bevy::input::InputPlugin::default());
    app.add_plugin(bevy::core_pipeline::CorePipelinePlugin::default());
    app.add_plugin(bevy::transform::TransformPlugin::default());
    app.add_plugin(bevy::sprite::SpritePlugin::default());

    // events

    // systems
    app.add_startup_system(building_meshes);

    // run
    app.run();
}

fn building_meshes(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(bevy::prelude::Camera2dBundle {
        transform: bevy::prelude::Transform::from_xyz(0.0, 0.0, 999.9),
        ..Default::default()
    });

    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/only_point.geojson".to_owned()));

    //proj instances
    let from = "EPSG:4326";
    let to = "EPSG:3875";

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();

        // convert geom to proj format
        let proj = Proj::new_known_crs(&from, &to, None).unwrap();
        let result = projection_geometry(geom.clone().into(), proj);

        println!("coord before projection: {:?}", geom);
        println!("coord after projection: {:?}", result);

        let mesh_iter = build_bevy_meshes(&geom, Color::RED, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();

        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::GeometryMesh::Point(points) => {
                    for geo::Point(coord) in points.iter() {
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
                geo_bevy::GeometryMesh::Polygon { mesh, color } => {
                    println!("Inside Polygon, topology:{:?}", mesh.primitive_topology());
                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: bevy::prelude::Transform::from_xyz(0., 0., 0.),
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                }
                geo_bevy::GeometryMesh::LineString{mesh, color} => {
                    println!("Inside LineString, topology:{:?}", mesh.primitive_topology());
                    let material = materials.add(color.into());

                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh)),
                        transform: bevy::prelude::Transform::from_xyz(0., 0., 0.),
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