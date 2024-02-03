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
use bevy_prototype_lyon::prelude::*;
use geo::Centroid;
use geo_types::Geometry;
use gis_layers::AllLayers;
use gis_test::*;

#[derive(Component, Clone)]
struct Files {
    file_name: String,
    file_path: String,
    file_layers: AllLayers,
    file_entities: Vec<Entity>,
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
    mut files_query: Query<&Files>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    entities_query: Query<Entity, Without<Camera>>,
    camera_query: Query<Entity, With<Camera>>,
) {
    let camera = camera_query.get_single().unwrap();
    egui::SidePanel::left("main")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(RichText::new("GIS").color(Color32::RED).strong());
                ui.separator();
                let select_btn =
                    egui::Button::new(RichText::new("▶ Select File").color(Color32::GREEN));
                let clear_btn = egui::Button::new(RichText::new("▶ Clear").color(Color32::YELLOW));
                let exit_btn = egui::Button::new(RichText::new("▶ Exit").color(Color32::RED));

                if ui.add(select_btn).clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let file_path = Some(path.display().to_string()).unwrap();
                        let file_name = path.file_name().unwrap().to_str().unwrap();

                        let (layers, entities) = build_meshes(
                            &mut *meshes,
                            &mut *materials,
                            &mut commands,
                            file_path.to_owned(),
                            file_name.to_owned(),
                        );
                        let files = Files {
                            file_name: file_name.to_owned(),
                            file_path: file_path.to_owned(),
                            file_layers: layers,
                            file_entities: entities,
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

                ui.separator();

                for file in files_query.iter_mut() {
                    let file_name = &file.file_name;
                    let add_file_btn =
                        egui::Button::new(RichText::new("Add").color(Color32::WHITE));
                    let remove_file_btn =
                        egui::Button::new(RichText::new("Remove").color(Color32::WHITE));
                    let label_text = file_name.to_owned() + " actions";

                    ui.label(RichText::new(label_text).strong());

                    if ui.add(add_file_btn).clicked() {
                        let (_, _) = build_meshes(
                            &mut *meshes,
                            &mut *materials,
                            &mut commands,
                            file.file_path.to_owned(),
                            file_name.to_owned(),
                        );
                    }

                    if ui.add(remove_file_btn).clicked() {
                        despawn_entities_file(&mut commands, file.clone());
                    }

                    ui.separator();
                }
            })
        });
}

fn build_meshes(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    commands: &mut Commands,
    file_path: String,
    file_name: String,
) -> (AllLayers, Vec<Entity>) {
    let geojson = read_geojson(file_path);
    let feature_collection = read_geojson_feature_collection(geojson);
    let mut layers: gis_layers::AllLayers = gis_layers::AllLayers::new();
    let mut entities_id: Vec<Entity> = Vec::new();

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo::Geometry = geometry.try_into().unwrap();

        match geom {
            Geometry::Polygon(polygon) => {
                layers.add(geo::Geometry::Polygon(polygon.clone()), file_name.clone());

                let (builder, transform) = build_polygon(polygon, layers.last_layer_id());

                let id = commands
                    .spawn(builder.build(
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::WHITE),
                            outline_mode: StrokeMode::new(Color::BLUE, 0.1),
                        },
                        transform,
                    ))
                    .id();

                entities_id.push(id);
            }
            Geometry::LineString(linestring) => {
                layers.add(
                    geo::Geometry::LineString(linestring.clone()),
                    file_name.clone(),
                );

                let (builder, transform) = build_linestring(linestring, layers.last_layer_id());

                let id = commands
                    .spawn(builder.build(
                        DrawMode::Stroke(StrokeMode::color(Color::YELLOW_GREEN)),
                        transform,
                    ))
                    .id();

                entities_id.push(id);
            }
            Geometry::Point(point) => {
                let center = point.centroid();
                layers.add(geom.clone(), file_name.clone());
                let z = calculate_z(layers.last_layer_id(), MeshType::Point);

                let id = commands
                    .spawn(bevy::sprite::MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                        material: materials.add(Color::PINK.into()),
                        transform: Transform::from_translation(Vec3::new(
                            center.0.x as f32,
                            center.0.y as f32,
                            z,
                        )),
                        ..Default::default()
                    })
                    .id();

                entities_id.push(id);
            }
            Geometry::MultiPolygon(multi_polygon) => {
                layers.add(
                    geo::Geometry::MultiPolygon(multi_polygon.clone()),
                    file_name.clone(),
                );

                for polygon in multi_polygon.0.iter() {
                    let (builder, transform) =
                        build_polygon(polygon.clone(), layers.last_layer_id());

                    let id = commands
                        .spawn(builder.build(
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::WHITE),
                                outline_mode: StrokeMode::new(Color::BLUE, 0.1),
                            },
                            transform,
                        ))
                        .id();

                    entities_id.push(id);
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

                    let id = commands
                        .spawn(builder.build(
                            DrawMode::Stroke(StrokeMode::color(Color::YELLOW_GREEN)),
                            transform,
                        ))
                        .id();

                    entities_id.push(id);
                }
            }
            Geometry::MultiPoint(multi_point) => {
                for point in multi_point {
                    layers.add(geo::Geometry::Point(point), file_name.clone());
                    let z = calculate_z(layers.last_layer_id(), MeshType::Point);
                    let id = commands
                        .spawn(bevy::sprite::MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Circle::new(1.).into()).into(),
                            material: materials.add(Color::PINK.into()),
                            transform: Transform::from_translation(Vec3::new(
                                point.0.x as f32,
                                point.0.y as f32,
                                z,
                            )),
                            ..Default::default()
                        })
                        .id();

                    entities_id.push(id);
                }
            }
            _ => continue,
        }
    }

    (layers, entities_id)
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

fn despawn_entities_file(commands: &mut Commands, file: Files) {
    for entity in file.file_entities.iter() {
        commands.entity(*entity).despawn();
    }
}
