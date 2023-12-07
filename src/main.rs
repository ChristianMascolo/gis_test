#![allow(deprecated)]

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
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle}, ecs::{world::World, system::{Query, Resource}, query::With, component::Component, reflect::ReflectComponent, entity::Entity}, math::{Vec2, Vec3, Mat4}, winit::UpdateMode, window::{Window, Windows}, transform::components::{GlobalTransform, Transform}, reflect::{Reflect, std_traits::ReflectDefault}, render::camera::{CameraProjection, OrthographicProjection}, core_pipeline::core_2d::Camera2dBundle,
};

use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_mod_raycast::*;

use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use gis_test::{read_geojson, read_geojson_feature_collection};

enum MeshType{
    Point,
    Polygon,
    LineString
}

#[derive(Component)]
struct Follow;

fn main() {
    let mut app = App::new();    

    // resources
    app.insert_resource(ClearColor(Color::rgb(255., 255., 255.)));

    // plugins
    app.add_plugins(bevy::MinimalPlugins);
    app.add_plugin(WindowPlugin {
        window: WindowDescriptor {
            width: 1100.,
            height: 900.,
            title: "gis_test".to_string(),
            ..Default::default()
        },
        ..Default::default()
    });
    app.add_plugin(DefaultRaycastingPlugin::<()>::default());
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
    app.add_plugin(bevy_inspector_egui::DefaultInspectorConfigPlugin); // adds default options and `InspectorEguiImpl`s
    
    
    // systems
    app.add_startup_system(setup);
    app.add_system(inspector_ui);
    app.add_system(follow_entities);
    // run
    app.run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut layers = gis_layers::AllLayers::new();
    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/only_polygon.geojson".to_owned()));
    let mut last_id = 0;
    let mut i = 0;

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();

        let mesh_iter = build_bevy_meshes(&geom, Color::RED, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();
                        
        
        let _ = layers.add(geom, "mesh".to_owned());
        
        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::PreparedMesh::Point(points) => {
                     for geo::Point(coord) in points.iter() {
                        let color = Color::RED;
                        last_id = layers.last_layer_id();
                        let z_index = calculate_z(last_id, MeshType::Point) + i;
                        let transform =
                            bevy::prelude::Transform::from_xyz(coord.x as f32, coord.y as f32, z_index as f32);

                        let bundle = SpriteBundle {
                            sprite: Sprite {
                                color,
                                ..Default::default()
                            },
                            texture: asset_server.load("circle.png"),
                            transform,
                            ..Default::default()
                        };

                        commands.spawn(bundle).insert(Follow);
                        i=i+1;
                    }
                },

                geo_bevy::PreparedMesh::Polygon { mesh, color } => {
                    let material = materials.add(color.into());
                    last_id = layers.last_layer_id();
                    let z_index = calculate_z(last_id, MeshType::Polygon);
                    let transform = bevy::prelude::Transform::from_translation(Vec3::new(0., 0., z_index  as f32));
                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: transform,
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    }).insert(Follow);
                },

                geo_bevy::PreparedMesh::LineString{mesh, color} => {
                    let material = materials.add(color.into());
                    last_id = layers.last_layer_id();
                    let z_index = calculate_z(last_id, MeshType::LineString);
                    let transform = bevy::prelude::Transform::from_translation(Vec3::new(0., 0., z_index  as f32));
                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: transform,
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    }).insert(Follow);
                },
            }
        }
    }

    let global_transform = GlobalTransform::from_xyz(158 as f32, 68 as f32, 999.9);

    commands.spawn(Camera2dBundle{
        projection: OrthographicProjection{
            scale: -0.1,
            ..Default::default()
        },
        global_transform: global_transform,
        ..Default::default()
    });


}

fn calculate_z(layer_index: i32, mesh_type: MeshType) -> i32{
    return layer_index * 3
            + match mesh_type {
                MeshType::Point => 1,
                MeshType::Polygon => 2,
                MeshType::LineString => 3,
            }
}

fn inspector_ui(world: &mut World) {
    let egui_context = world.resource_mut::<bevy_inspector_egui::bevy_egui::EguiContext>().ctx_mut().clone();

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

            ui.heading("Entities");
            bevy_inspector::ui_for_world_entities(world, ui);
        });
    });
}

fn follow_entities(mut query:Query<(Entity,&Transform,&mut Transform),With<Follow>>){
    for (entity,transform, mut camera_transform) in query.iter_mut(){
        let target_position = transform.translation;
        camera_transform.translation = target_position + Vec3::new(0.0,0.0,10.0);
    }
}