#![allow(deprecated)]

mod gis_layer_id;
mod gis_layers;

use ::bevy::{
    prelude::{App, Color},
    window::{WindowDescriptor, WindowPlugin},
};

use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    math::{Vec2, Vec3},
    prelude::{ClearColor, Commands},
    transform::components::Transform,
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_prototype_lyon::prelude::*;

use geo::{Centroid, CoordsIter};
use geo_types::{Geometry, Point};
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

fn setup(mut commands: Commands) {
    let mut layers = gis_layers::AllLayers::new();
    let feature_collection = read_geojson_feature_collection(read_geojson(
        "C:/Users/masco/gis_test-1/maps/ikea_polygon                     .geojson".to_owned(),
    ));
    let mut camera_bundle = Camera2dBundle::default();

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();

        match geom {
            Geometry::Polygon(_) => {
                let mut coords = Vec::new();
                let z = calculate_z(layers.last_layer_id(), MeshType::Polygon);

                for coord in geom.coords_iter() {
                    let logical_coords = camera_bundle.camera.to_logical(Vec2::new(coord.x, coord.y));
                    
                    println!("non normalizzate -> x:{:?} y:{:?}",coord.x,coord.y);
                    println!("logic coord -> x:{:?} y:{:?}",logical_coords.x,logical_coords.y);

                    coords.push((logical_coords.x , logical_coords.x).into());
                }

                let shape = bevy_prototype_lyon::shapes::Polygon {
                    points: coords,
                    closed: true,
                };

                let builder = GeometryBuilder::new().add(&shape);
                let _ = layers.add(geom.clone(), "Polygon".to_owned());
                let translation = Vec3 {
                    x: 0.,
                    y: 0.,
                    z: z,
                };

                commands.spawn(GeometryBuilder::build_as(
                    &shape,
                    DrawMode::Stroke(StrokeMode::color(Color::ORANGE_RED)),
                    Transform::from_translation(translation),
                ));
            }
            Geometry::LineString(_) => {
                let mut coords: Vec<Point> = Vec::new();

                for coord in geom.coords_iter() {
                    coords.push(Point::new(coord.x as f64, coord.y as f64))
                }

                let start = coords.get(0).unwrap();
                let last = coords.last().unwrap();

                let shape = shapes::Line(
                    Vec2 {
                        x: start.0.x as f32,
                        y: start.0.y as f32,
                    },
                    Vec2 {
                        x: last.0.x as f32,
                        y: last.0.y as f32,
                    },
                );

                let builder = GeometryBuilder::new().add(&shape);
                let _ = layers.add(geom.clone(), "line_string".to_owned());
                let translation = Vec3 {
                    x: 0.,
                    y: 0.,
                    z: calculate_z(layers.last_layer_id(), MeshType::LineString),
                };

                commands.spawn(builder.build(
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::ORANGE_RED),
                        outline_mode: StrokeMode::new(Color::ORANGE_RED, 0.1),
                    },
                    Transform::from_translation(translation),
                ));
            }
            Geometry::Point(_) => {
                let centroid = geom.centroid().unwrap();
                let shape = shapes::Circle {
                    radius: 0.5,
                    center: Vec2::new(centroid.x() as f32, centroid.y() as f32),
                };
                let builder = GeometryBuilder::new().add(&shape);
                let _ = layers.add(geom.clone(), "point(s)".to_owned());
                let z = calculate_z(layers.last_layer_id(), MeshType::Point);

                commands.spawn(builder.build(
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::ORANGE_RED),
                        outline_mode: StrokeMode::color(Color::ORANGE_RED),
                    },
                    Transform::from_xyz(0., 0. , z),
                ));
            }
            Geometry::Line(_) => {
                let mut coords = Vec::new();

                for coord in geom.coords_iter() {
                    coords.push((coord.x as f32, coord.y as f32).into())
                }

                let shape = bevy_prototype_lyon::shapes::Polygon {
                    points: coords,
                    closed: false,
                };

                let translation = Vec3 {
                    x: 0.,
                    y: 0.,
                    z: calculate_z(layers.last_layer_id(), MeshType::Polygon) as f32,
                };

                commands.spawn(GeometryBuilder::build_as(
                    &shape,
                    DrawMode::Fill(FillMode::color(Color::CYAN)),
                    Transform::from_translation(translation),
                ));
            }
            Geometry::MultiPoint(_) => todo!(),
            Geometry::MultiLineString(_) => todo!(),
            Geometry::MultiPolygon(_) => todo!(),
            Geometry::GeometryCollection(_) => todo!(),
            Geometry::Rect(_) => todo!(),
            Geometry::Triangle(_) => todo!(),
        }
    }

    let center = medium_centroid(get_all_centroids(layers));

    camera_bundle.projection = bevy::render::camera::OrthographicProjection {
        near: 0.,
        far: 1000.,
        scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
        scale: 0.1,
        ..Default::default()
    };

    camera_bundle.transform = Transform::from_xyz(center.0.x as f32, center.0.y as f32, 999.9);

    commands.spawn(camera_bundle);
}

fn calculate_z(layer_index: i32, mesh_type: MeshType) -> f32 {
    return layer_index as f32 * 3.
        + match mesh_type {
            MeshType::Point => 1.,
            MeshType::Polygon => 2.,
            MeshType::LineString => 3.,
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

fn get_all_centroids(layers: AllLayers) -> Vec<Point> {
    let mut centroids: Vec<Point> = Vec::new();

    for layer in layers.iter() {
        let geom = &layer.geom_type;
        centroids.push(geom.centroid().unwrap());
    }

    centroids
}

fn medium_centroid(centroids: Vec<Point>) -> Point {
    let mut somma_x = 0.0;
    let mut somma_y = 0.0;

    for centroid in centroids.clone() {
        somma_x += centroid.0.x;
        somma_y += centroid.0.y;
    }

    Point::new(
        somma_x / centroids.len() as f64,
        somma_y / centroids.len() as f64,
    )
}

//fn build_line_string(points: Vec<Point>, geom_builder: Geom) -> GeometryBuilder{
//    let mut i = 0;
//    let mut geom_builder = GeometryBuilder::new();
//
//    for point in points.clone(){
//        if (i + 1) == points.len() { break; }
//
//        let start = points.get(i).unwrap();
//        let last = points.get(i+1).unwrap();
//
//        let shape = shapes::Line(
//            Vec2 {
//                x: start.x() as f32,
//                y: start.y() as f32,
//            },
//            Vec2 {
//                x: last.x() as f32,
//                y: last.y() as f32,
//            },
//        );
//
//        i += 1;
//
//        geom_builder.add(&shape);
//    }
//
//    geom_builder
//}
