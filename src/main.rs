mod gis_layer_id;
mod gis_layers;

use std::ops::Deref;

use ::bevy::{prelude::*, window::Window};

use bevy_pancam::*;

use bevy::render::camera::ScalingMode;
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts, EguiPlugin,
};
use bevy::winit::WinitWindows;
// use bevy_math::primitives::dim2::Circle;
use bevy::math::primitives::Circle;
// use bevy_pancam::PanCam;
use bevy_prototype_lyon::draw::{Fill, Stroke};
use bevy_prototype_lyon::prelude::*;
use geo::Centroid;
use geo_types::Geometry;
use gis_layers::AllLayers;
use gis_test::*;
use rfd::*;

#[derive(Component, Clone)]
struct EntityFile {
    name: String,
    path: String,
    layers: AllLayers,
    entities: Vec<Entity>,
}

fn main() {
    let mut app = App::new();

    // resources
    app.insert_resource(ClearColor(Color::BLACK));

    // plugins
    app.add_plugins((bevy::DefaultPlugins,PanCamPlugin::default()));
    app.add_plugins(ShapePlugin);
    app.add_plugins(EguiPlugin);
    // systems
    app.add_systems(Startup, startup);
    app.add_systems(Update, ui);

    // run
    app.run();
}

fn startup(mut commands: Commands) {
    let far = 1000.;
    // Offset the whole simulation to the left to take the width of the UI panel into account.
    let ui_offset = -700.;
    // Scale the simulation so it fills the portion of the screen not covered by the UI panel.
    let scale_x = 700. / (700. + ui_offset);
    // The translation x must depend on the scale_x to keep the left offset constant between window resizes.
    let mut initial_transform = Transform::from_xyz(ui_offset * scale_x, 0., far - 0.1);
    initial_transform.scale.x = scale_x;
    initial_transform.scale.y = 700. / 700.;

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            far,
            scaling_mode: ScalingMode::WindowSize(1.),
            viewport_origin: Vec2::new(0., 0.),
            ..default()
        }
        .into(),
        transform: initial_transform,
        ..default()
    }).insert(PanCam::default());
}

fn ui(
    mut egui_context: EguiContexts,
    mut commands: Commands,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut files_query: Query<&EntityFile>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    file_entity_query: Query<Entity, With<EntityFile>>,
    all_entities_query: Query<Entity, Without<Camera>>,
    camera_query: Query<Entity, With<Camera>>,
    mut windows: NonSend<WinitWindows>
) {
    if let Some(camera) = camera_query.get_single().ok() {
        egui::SidePanel::left("main")
            // .resizable(true)
            .show(egui_context.ctx_mut(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("GIS").color(Color32::RED).strong());
                    ui.separator();
                    let select_btn =
                        egui::Button::new(RichText::new("▶ Select File").color(Color32::GREEN));
                    let clear_btn =
                        egui::Button::new(RichText::new("▶ Clear").color(Color32::YELLOW));
                    let exit_btn = egui::Button::new(RichText::new("▶ Exit").color(Color32::RED));

                    if ui.add(select_btn).clicked() {
                        if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
                            let extension = path_buf.extension().unwrap();
                            if extension.eq("json") || extension.eq("geojson") {
                                let path = Some(path_buf.display().to_string()).unwrap();
                                let name = path_buf.file_name().unwrap().to_str().unwrap();
                                let (layers, entities) = build_meshes(
                                    &mut *meshes,
                                    &mut *materials,
                                    &mut commands,
                                    path.to_owned(),
                                    name.to_owned(),
                                );
                                let entity_file = EntityFile {
                                    name: name.to_owned(),
                                    path: path.to_owned(),
                                    layers: layers,
                                    entities: entities,
                                };
                                let mut vec_entity_file: Vec<EntityFile> = Vec::new();

                                vec_entity_file.push(entity_file.clone());

                                commands.spawn(entity_file);

                                for file in files_query.iter() {
                                    vec_entity_file.push(file.clone());
                                }

                                center_camera(&mut commands, camera, vec_entity_file);
                            }
                        }
                    }

                    if ui.add(clear_btn).clicked() {
                        for entity in all_entities_query.iter() {
                            commands.entity(entity).despawn();
                        }
                    }

                    if ui.add(exit_btn).clicked() {
                        app_exit_events.send(bevy::app::AppExit);
                    }

                    ui.separator();

                    for file in &mut files_query.iter_mut() {
                        let name = &file.name;
                        let remove_file_btn =
                            egui::Button::new(RichText::new("Remove").color(Color32::WHITE));
                        let label_text = name.to_owned();

                        ui.label(
                            RichText::new(label_text)
                                .strong()
                                .color(Color32::DEBUG_COLOR),
                        );

                        if ui.add(remove_file_btn).clicked() {
                            for entity_file in file_entity_query.iter() {
                                commands.entity(entity_file).despawn();
                            }

                            for entity in file.entities.iter() {
                                commands.entity(*entity).despawn();
                            }
                        }

                        ui.separator();
                    }
                })
            });
    }
}

fn build_meshes(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    commands: &mut Commands,
    path: String,
    name: String,
) -> (AllLayers, Vec<Entity>) {
    let geojson = read_geojson(path);
    let feature_collection = read_geojson_feature_collection(geojson);
    let mut layers: gis_layers::AllLayers = gis_layers::AllLayers::new();
    let mut entities_id: Vec<Entity> = Vec::new();
    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo::Geometry = geometry.try_into().unwrap();
        match geom {
            Geometry::Polygon(polygon) => {
                layers.add(geo::Geometry::Polygon(polygon.clone()), name.clone());

                let (builder, transform) = build_polygon(polygon, layers.last_layer_id());


                let id = commands
                    .spawn((
                        ShapeBundle {
                            path: builder.build(),
                            ..default()
                        },
                        Fill::color(Color::WHITE),
                        Stroke::new(Color::BLUE, 0.1),
                        // transform,
                    ))
                    .id();

                entities_id.push(id);
            }
            Geometry::LineString(linestring) => {
                layers.add(geo::Geometry::LineString(linestring.clone()), name.clone());

                let (builder, transform) = build_linestring(linestring, layers.last_layer_id());

                let id = commands
                    .spawn((
                        ShapeBundle {
                            path: builder.build(),
                            ..default()
                        },
                        // builder.build(),
                        Fill::color(Color::RED),
                        Stroke::new(Color::YELLOW_GREEN, 0.1),
                        // transform,
                    ))
                    .id();
                
                entities_id.push(id);
            }
            Geometry::Point(point) => {
                let center = point.centroid();
                layers.add(geom.clone(), name.clone());
                let z = calculate_z(layers.last_layer_id(), MeshType::Point);

                let id = commands
                    .spawn(bevy::sprite::MaterialMesh2dBundle {
                        // mesh: meshes.add(Circle::new(1.).into()).into(),
                        mesh: bevy::sprite::Mesh2dHandle(meshes.add(Circle::new(1.).mesh())),
                        material: materials.add(Color::PINK),
                        // transform: Transform::from_translation(Vec3::new(
                        //     center.0.x as f32,
                        //     center.0.y as f32,
                        //     z,
                        // )),
                        ..Default::default()
                    })
                    .id();

                entities_id.push(id);
            }
            Geometry::MultiPolygon(multi_polygon) => {
                layers.add(
                    geo::Geometry::MultiPolygon(multi_polygon.clone()),
                    name.clone(),
                );

                for polygon in multi_polygon.0.iter() {
                    let (builder, transform) =
                        build_polygon(polygon.clone(), layers.last_layer_id());

                    let id = commands
                        .spawn((
                            ShapeBundle {
                                path: builder.build(),
                                ..default()
                            },
                            // builder.build(),
                            Fill::color(Color::WHITE),
                            Stroke::new(Color::BLUE, 0.1),
                            // transform,
                        ))
                        .id();
                    entities_id.push(id);
                }
            }
            Geometry::MultiLineString(multi_line_string) => {
                layers.add(
                    geo::Geometry::MultiLineString(multi_line_string.clone()),
                    name.clone(),
                );

                for line in multi_line_string.iter() {
                    let (builder, transform) =
                        build_linestring(line.clone(), layers.last_layer_id());

                    let id = commands
                        .spawn((
                            // builder.build(),
                            ShapeBundle {
                                path: builder.build(),
                                ..default()
                            },
                            Fill::color(Color::WHITE),
                            Stroke::new(Color::YELLOW_GREEN, 0.1),
                            // transform,
                        ))
                        .id();

                    entities_id.push(id);
                }
            }
            Geometry::MultiPoint(multi_point) => {
                for point in multi_point {
                    layers.add(geo::Geometry::Point(point), name.clone());
                    let z = calculate_z(layers.last_layer_id(), MeshType::Point);
                    let id = commands
                        .spawn(bevy::sprite::MaterialMesh2dBundle {
                            mesh: bevy::sprite::Mesh2dHandle(meshes.add(Circle::new(1.).mesh())),
                            material: materials.add(Color::PINK),
                            // transform: Transform::from_translation(Vec3::new(
                            //     point.0.x as f32,
                            //     point.0.y as f32,
                            //     z,
                            // )),
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

fn center_camera(commands: &mut Commands, camera: Entity, entity_file: Vec<EntityFile>) {
    let mut points: Vec<geo_types::Point<f64>> = Vec::new();
    let mut new_camera = Camera2dBundle::default();

    commands.entity(camera).despawn();

    for file in entity_file {
        let layers = file.layers;
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
