#![allow(deprecated)]

mod gis_layer_id;
mod gis_layers;

use ::bevy::{
    prelude::{App, Color},
    window::{WindowDescriptor, WindowPlugin},
};

use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    math::{Vec3, Vec2},
    prelude::{AssetServer, Assets, ClearColor, Commands, Mesh, Res, ResMut},
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle},
    transform::components::Transform,
    window::{Windows, WindowMode}, render::camera::Camera, input::mouse::MouseButton, ecs::system::Query,
};
use bevy_prototype_lyon::{prelude::*, entity::ShapeBundle};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;

use geo::{Centroid, CoordsIter};
use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use geo_types::{Point, Geometry, Coord};
use gis_layers::AllLayers;
use gis_test::{read_geojson, read_geojson_feature_collection};

enum MeshType {
    Point,
    Polygon,
    LineString,
}

fn main() {
    let mut app = App::new();

    // resources
    app.insert_resource(ClearColor(Color::rgb(255., 255., 255.)));

    // plugins
    app.add_plugins(bevy::MinimalPlugins);
    app.add_plugin(WindowPlugin {
        window: WindowDescriptor {
            title: "gis_test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    });
    app.add_plugin(bevy::asset::AssetPlugin::default());
    app.add_plugin(bevy::winit::WinitPlugin::default());
    app.add_plugin(bevy::render::RenderPlugin::default());
    app.add_plugin(bevy::render::texture::ImagePlugin::default());
    app.add_plugin(bevy::log::LogPlugin::default());
    app.add_plugin(bevy::input::InputPlugin::default());
    app.add_plugin(bevy::core_pipeline::CorePipelinePlugin::default());
    app.add_plugin(bevy::transform::TransformPlugin::default());
    app.add_plugin(bevy::sprite::SpritePlugin::default());
    app.add_plugin(EguiPlugin);
    app.add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin);
    app.add_plugin(ShapePlugin);

    // systems
    app.add_startup_system(setup);
    app.add_system(inspector_ui);

    // run
    app.run();
}

fn setup(
    asset_server: Res<AssetServer>,
    windows: ResMut<Windows>,
    mut commands: Commands,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut layers = gis_layers::AllLayers::new();
    let feature_collection =
        read_geojson_feature_collection(read_geojson("C:/Users/masco/gis_test-1/maps/all_geometry.geojson".to_owned()));
    let mut i = 0;
    let primary_window = windows.get_primary().unwrap();

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();
        //let mesh_iter = build_bevy_meshes(&geom, Color::RED, BuildBevyMeshesContext::new())
        //    .unwrap()
        //    .collect::<Vec<_>>();

        let _ = layers.add(geom.clone(), "mesh".to_owned());
        
        match geom{
            Geometry::Polygon(_) => {
                let mut coords = Vec::new();
    
                for coord in geom.coords_iter(){ coords.push((coord.x as f32,coord.y as f32).into()) }
                
                let shape = bevy_prototype_lyon::shapes::Polygon{
                    points: coords,
                    closed: true,
                };

                let translation = Vec3 { x: 0. , y: 0., z: calculate_z(layers.last_layer_id(), MeshType::Polygon) as f32 };

                commands.spawn(
                        GeometryBuilder::build_as(&shape,DrawMode::Fill(FillMode::color(Color::CYAN)),Transform::from_translation(translation)),
                );

            },
            Geometry::LineString(_) => {
                let mut coords = Vec::new();
    
                for coord in geom.coords_iter(){ coords.push((coord.x as f32,coord.y as f32).into()) }
                
                let shape = bevy_prototype_lyon::shapes::Polygon{
                    points: coords,
                    closed: false,
                };

                let translation = Vec3 { x: 0. , y: 0., z: calculate_z(layers.last_layer_id(), MeshType::Polygon) as f32 };

                commands.spawn(
                        GeometryBuilder::build_as(&shape,DrawMode::Fill(FillMode::color(Color::CYAN)),Transform::from_translation(translation)),
                );
            },
            Geometry::Point(_) =>{
                let shape = shapes::Circle{
                    radius: 1.,
                    center: Vec2::new(geom.centroid().unwrap().x()as f32 ,geom.centroid().unwrap().y() as f32),
                };

                let translation = Vec3 { x: 0. , y: 0., z: calculate_z(layers.last_layer_id(), MeshType::Point) as f32 };

                commands.spawn(
                        GeometryBuilder::build_as(&shape,DrawMode::Fill(FillMode::color(Color::CYAN)),Transform::from_translation(translation)),
                );

            },
            Geometry::Line(_) => {
                let mut coords = Vec::new();
    
                for coord in geom.coords_iter(){ coords.push((coord.x as f32,coord.y as f32).into()) }
                
                let shape = bevy_prototype_lyon::shapes::Polygon{
                    points: coords,
                    closed: false,
                };

                let translation = Vec3 { x: 0. , y: 0., z: calculate_z(layers.last_layer_id(), MeshType::Polygon) as f32 };

                commands.spawn(
                        GeometryBuilder::build_as(&shape,DrawMode::Fill(FillMode::color(Color::CYAN)),Transform::from_translation(translation)),
                );
            },
            Geometry::MultiPoint(_) => todo!(),
            Geometry::MultiLineString(_) => todo!(),
            Geometry::MultiPolygon(_) => todo!(),
            Geometry::GeometryCollection(_) => todo!(),
            Geometry::Rect(_) => todo!(),
            Geometry::Triangle(_) => todo!(),
        }

        //for prepared_mesh in mesh_iter {
        //    match prepared_mesh {
        //        geo_bevy::PreparedMesh::Point(points) => {
        //            for geo::Point(coord) in points.iter() {
        //                let color = Color::RED;
        //                let last_id = layers.last_layer_id();
        //                let z_index = calculate_z(last_id, MeshType::Point) + i;
        //                let mut transform = bevy::prelude::Transform::from_xyz(
        //                    coord.x as f32,
        //                    coord.y as f32,
        //                    z_index as f32,
        //                );
//
        //                let bundle = SpriteBundle {
        //                    sprite: Sprite {
        //                        color,
        //                        custom_size: Some(bevy::math::Vec2::new(1.,1.)),
        //                        ..Default::default()
        //                    },
        //                    texture: asset_server.load("circle.png"),
        //                    transform,
        //                    ..Default::default()
        //                };
//
        //                commands.spawn(bundle);
        //                i = i + 1;
        //            }
        //        }
//
        //        //geo_bevy::PreparedMesh::Polygon { mesh, color } => {
        //        //    let last_id = layers.last_layer_id();
        //        //    let material = materials.add(color.into());
        //        //    let z_index = calculate_z(last_id, MeshType::Polygon);
        //        //    let transform = bevy::prelude::Transform::from_translation(Vec3::new(
        //        //        0.,
        //        //        0.,
        //        //        z_index as f32,
        //        //    ));
////
////
        //        //    commands
        //        //        .spawn(MaterialMesh2dBundle {
        //        //            material,
        //        //            mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
        //        //            transform: transform,
        //        //            visibility: bevy::render::view::Visibility { is_visible: true },
        //        //            ..Default::default()
        //        //        });
        //        //}
//
        //        geo_bevy::PreparedMesh::LineString { mesh, color } => {
        //            let last_id = layers.last_layer_id();
        //            let material = materials.add(color.into());
        //            let z_index = calculate_z(last_id, MeshType::LineString);
        //            let transform = bevy::prelude::Transform::from_translation(Vec3::new(
        //                0.,
        //                0.,
        //                z_index as f32,
        //            ));
//
        //            commands
        //                .spawn(MaterialMesh2dBundle {
        //                    material,
        //                    mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
        //                    transform: transform,
        //                    visibility: bevy::render::view::Visibility { is_visible: true },
        //                    ..Default::default()
        //                });
        //        }
        //        geo_bevy::PreparedMesh::Polygon { mesh, color } => todo!(),
        //    }
        //}

        }

        commands.spawn(create_camera(get_all_centroids(layers), primary_window.width(), primary_window.height()));
}

fn calculate_z(layer_index: i32, mesh_type: MeshType) -> i32 {
    return layer_index * 3
        + match mesh_type {
            MeshType::Point => 1,
            MeshType::Polygon => 2,
            MeshType::LineString => 3,
        };
}

fn inspector_ui(world: &mut bevy::ecs::world::World) {
    let egui_context = world
        .resource_mut::<bevy_inspector_egui::bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    bevy_inspector_egui::egui::Window::new("UI").show(&egui_context, |ui| {
        bevy_inspector_egui::egui::ScrollArea::vertical().show(ui, |ui| {
            // equivalent to `WorldInspectorPlugin`
            bevy_inspector::ui_for_world(world, ui);

            // works with any `Reflect` value, including `Handle`s
            let mut any_reflect_value: i32 = 5;
            bevy_inspector::ui_for_value(&mut any_reflect_value, ui, world);

            bevy_inspector_egui::egui::CollapsingHeader::new("Materials").show(ui, |ui| {
                bevy_inspector::ui_for_assets::<bevy::pbr::StandardMaterial>(world, ui);
            });
        });
    });
}

fn get_all_centroids(layers: AllLayers) -> Vec<Point>{
    let mut centroids: Vec<Point> = Vec::new();

    for layer in layers.iter(){
        let geom = &layer.geom_type;
        centroids.push(geom.centroid().unwrap());
    }

    centroids
}

fn create_camera(centroids: Vec<Point>, win_width: f32, win_height: f32) -> Camera2dBundle{
    let center = medium_centroid(centroids);

    Camera2dBundle {
        projection: bevy::render::camera::OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: 0.1,
            ..Default::default()
        },
        transform: Transform::from_xyz(center.0.x as f32, center.0.y as f32, 999.9),
        ..Default::default()
    }
}

fn medium_centroid(centroids: Vec<Point>) -> Point{
    let mut somma_x = 0.0;
    let mut somma_y = 0.0;

    for centroid in centroids.clone(){
        somma_x += centroid.0.x;
        somma_y += centroid.0.y;
    }

    Point::new(somma_x / centroids.len() as f64, somma_y / centroids.len() as f64)
}