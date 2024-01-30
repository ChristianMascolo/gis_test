mod gis_layer_id;
mod gis_layers;

use ::bevy::{
    prelude::{App, Color},
    window::{WindowDescriptor, WindowPlugin},
};

use bevy::prelude::*;

use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_pancam::PanCam;
use bevy_prototype_lyon::prelude::*;
use eframe::{
    egui::{CentralPanel, Context},
    App as OtherApp, Frame,
};
use egui_file::FileDialog;
use geo::Centroid;
use geo_types::{Geometry, Point};
use gis_test::*;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Demo {
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
}

impl OtherApp for Demo {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            if (ui.button("Open")).clicked() {
                // Show only files with the extension "txt".
                let filter = Box::new({
                    let ext = Some(OsStr::new("txt"));
                    move |path: &Path| -> bool { path.extension() == ext }
                });
                let mut dialog =
                    FileDialog::open_file(self.opened_file.clone()).show_new_folder(true);
                dialog.open();
                self.open_file_dialog = Some(dialog);
            }

            if let Some(dialog) = &mut self.open_file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        self.opened_file = Some(file.to_path_buf());
                    }
                }
            }
        });
    }
}

fn main() {
    let mut app = App::new();
    //let _ = eframe::run_native(
    //    "File Dialog Demo",
    //    eframe::NativeOptions::default(),
    //    Box::new(|_cc| Box::new(Demo::default())),
    //);

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
    app.add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin);
    app.add_plugin(bevy_pancam::PanCamPlugin::default());
    app.add_plugin(EguiPlugin);
    app.add_plugin(ShapePlugin);

    // systems
    app.add_startup_system(setup);
    app.add_system(inspector_ui);

    // run
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let geojson = read_geojson("C:/Users/masco/gis_test-1/maps/campania.geojson".to_owned());
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

    commands
        .spawn(setup_camera(layers))
        .insert(PanCam::default());
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

fn setup_camera(layers: gis_layers::AllLayers) -> Camera2dBundle {
    let mut camera = Camera2dBundle::default();
    let mut centroids: Vec<Point> = Vec::new();

    for layer in layers.iter() {
        let geom = &layer.geom_type;
        centroids.push(geom.centroid().unwrap());
    }

    let center = medium_centroid(centroids);

    camera.projection = bevy::render::camera::OrthographicProjection {
        near: 0.,
        far: 1000.,
        scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
        scale: 0.5,
        ..Default::default()
    };

    camera.transform = bevy::transform::components::Transform::from_xyz(
        center.0.x as f32,
        center.0.y as f32,
        999.9,
    );

    camera
}
