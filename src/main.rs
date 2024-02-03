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
use geo_types::{Geometry, Point};
use gis_layers::AllLayers;
use gis_test::*;

#[derive(Component, Clone)]
struct Files {
    file_name: String,
    file_layers: AllLayers,
}

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
    mut commands: Commands,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    entities_query: Query<Entity, Without<Camera>>,
    camera_query: Query<Entity, With<Camera>>,
    files_query: Query<&Files>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = camera_query.get_single().unwrap();
    egui::SidePanel::left("main").show(egui_context.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("GIS");
            ui.separator();
            let select_btn =
                egui::Button::new(RichText::new("▶ Select File").color(Color32::GREEN));
            let clear_btn = egui::Button::new(RichText::new("▶ Clear").color(Color32::YELLOW));
            let exit_btn = egui::Button::new(RichText::new("▶ Exit").color(Color32::RED));

            if ui.add(select_btn).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    let file_path = Some(path.display().to_string()).unwrap();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let layers = build_meshes(
                        meshes,
                        materials,
                        &mut commands,
                        file_path,
                        file_name.to_owned(),
                    );
                    let files = Files {
                        file_name: file_name.to_owned(),
                        file_layers: layers,
                    };
                    let mut vec_files: Vec<Files> = Vec::new();

                    vec_files.push(files.clone());

                    commands.spawn(files);

                    for file in files_query.iter() {
                        vec_files.push(file.clone());
                    }

                    center_camera(&mut commands, camera, vec_files);
                }
            }

            if ui.add(clear_btn).clicked() {
                for entity in entities_query.iter() {
                    commands.entity(entity).despawn();
                }
            }

            if ui.add(exit_btn).clicked() {
                app_exit_events.send(bevy::app::AppExit);
            }
        })
    });
}

fn build_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands,
    file_path: String,
    file_name: String,
) -> AllLayers {
    let geojson = read_geojson(file_path);
    let feature_collection = read_geojson_feature_collection(geojson);
    let mut layers: gis_layers::AllLayers = gis_layers::AllLayers::new();

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo::Geometry = geometry.try_into().unwrap();

        match geom {
            Geometry::Polygon(polygon) => {
                layers.add(geo::Geometry::Polygon(polygon.clone()), file_name.clone());

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
                    file_name.clone(),
                );

                let (builder, transform) = build_linestring(linestring, layers.last_layer_id());

                commands.spawn(builder.build(
                    DrawMode::Stroke(StrokeMode::color(Color::YELLOW_GREEN)),
                    transform,
                ));
            }
            Geometry::Point(point) => {
                let center = point.centroid();
                layers.add(geom.clone(), file_name.clone());
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
                    file_name.clone(),
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
                    file_name.clone(),
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
                    layers.add(geo::Geometry::Point(point), file_name.clone());
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

    layers
}

fn center_camera(commands: &mut Commands, camera: Entity, files: Vec<Files>) {
    let mut points: Vec<geo_types::Point<f64>> = Vec::new();
    let mut new_camera = Camera2dBundle::default();

    commands.entity(camera).despawn();

    for file in files {
        let layers = file.file_layers;
        for layer in layers.iter() {
            let geom = &layer.geom_type;
            let centroid = geom.centroid().unwrap();

            points.push(centroid);
        }
    }

    let center = medium_centroid(points);

    new_camera.transform = Transform::from_xyz(center.0.x as f32, center.0.y as f32, 999.9);

    commands.spawn(new_camera).insert(PanCam::default());
}
