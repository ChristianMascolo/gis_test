mod gis_layer_id;
mod gis_layers;

use ::bevy::{
    prelude::*,
    window::{WindowDescriptor, WindowPlugin},
};

use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContext, EguiPlugin,
};

use bevy_pancam::PanCam;
use bevy_prototype_lyon::{entity, prelude::*};
use geo::Centroid;
use geo_types::Geometry;
use gis_test::*;

fn main() {
    let mut app = App::new();

    // resources
    app.insert_resource(ClearColor(Color::BLACK));

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
    app.add_plugin(bevy_pancam::PanCamPlugin::default());
    app.add_plugin(ShapePlugin);
    app.add_plugin(EguiPlugin);

    // systems
    app.add_startup_system(startup);
    app.add_system(ui);
    // run
    app.run();
}

fn startup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    egui::SidePanel::left("main").show(egui_context.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("GIS");
            ui.separator();
            let file_btn = egui::Button::new(RichText::new("▶ Select File").color(Color32::GREEN));

            if ui.add(file_btn).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let file_path = Some(path.display().to_string()).unwrap();
                    build_meshes(meshes, materials, commands, file_path);
                }
            }

        })
    });
}

fn build_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    file_path: String,
) {
    let geojson = read_geojson(file_path);
    let feature_collection = read_geojson_feature_collection(geojson);
    let mut layers: gis_layers::AllLayers = gis_layers::AllLayers::new();

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo::Geometry = geometry.try_into().unwrap();

        match geom {
            Geometry::Polygon(polygon) => {
                layers.add(
                    geo::Geometry::Polygon(polygon.clone()),
                    "Polygon".to_owned(),
                );

                let (builder, transform) = build_polygon(polygon, layers.last_layer_id());

                commands.spawn(builder.build(
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::WHITE),
                        outline_mode: StrokeMode::new(Color::BLUE, 0.1),
                    },
                    transform,
                ));
            }
            Geometry::LineString(linestring) => {
                layers.add(
                    geo::Geometry::LineString(linestring.clone()),
                    "line_string".to_owned(),
                );

                let (builder, transform) = build_linestring(linestring, layers.last_layer_id());

                commands.spawn(builder.build(
                    DrawMode::Stroke(StrokeMode::color(Color::YELLOW_GREEN)),
                    transform,
                ));
            }
            Geometry::Point(point) => {
                let center = point.centroid();
                layers.add(geom.clone(), "point(s)".to_owned());
                let z = calculate_z(layers.last_layer_id(), MeshType::Point);

                commands.spawn(bevy::sprite::MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                    material: materials.add(Color::PINK.into()),
                    transform: Transform::from_translation(Vec3::new(
                        center.0.x as f32,
                        center.0.y as f32,
                        z,
                    )),
                    ..Default::default()
                });
            }
            Geometry::MultiPolygon(multi_polygon) => {
                layers.add(
                    geo::Geometry::MultiPolygon(multi_polygon.clone()),
                    "MultiPolygon".to_owned(),
                );

                for polygon in multi_polygon.0.iter() {
                    let (builder, transform) =
                        build_polygon(polygon.clone(), layers.last_layer_id());

                    commands.spawn(builder.build(
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::WHITE),
                            outline_mode: StrokeMode::new(Color::BLUE, 0.1),
                        },
                        transform,
                    ));
                }
            }
            Geometry::MultiLineString(multi_line_string) => {
                layers.add(
                    geo::Geometry::MultiLineString(multi_line_string.clone()),
                    "MultiLineString".to_owned(),
                );

                for line in multi_line_string.iter() {
                    let (builder, transform) =
                        build_linestring(line.clone(), layers.last_layer_id());

                    commands.spawn(builder.build(
                        DrawMode::Stroke(StrokeMode::color(Color::YELLOW_GREEN)),
                        transform,
                    ));
                }
            }
            Geometry::MultiPoint(multi_point) => {
                for point in multi_point {
                    layers.add(geo::Geometry::Point(point), "Point".to_owned());
                    let z = calculate_z(layers.last_layer_id(), MeshType::Point);
                    commands.spawn(bevy::sprite::MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                        material: materials.add(Color::PINK.into()),
                        transform: Transform::from_translation(Vec3::new(
                            point.0.x as f32,
                            point.0.y as f32,
                            z,
                        )),
                        ..Default::default()
                    });
                }
            }
            _ => continue,
        }
    }
}
