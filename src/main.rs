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
    sprite::{ColorMaterial, MaterialMesh2dBundle, Sprite, SpriteBundle}, ecs::{world::World, system::{Query, Resource}, query::With, component::Component, reflect::ReflectComponent}, math::{Vec2, Vec3, Mat4}, winit::UpdateMode, window::{Window, Windows}, transform::components::GlobalTransform, reflect::{Reflect, std_traits::ReflectDefault}, render::camera::CameraProjection,
};

use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::bevy_inspector;
use bevy_mod_raycast::*;

use crate::{gis_layers::*, gis_event::MeshSpawnedEvent};
use crate::gis_camera::*;
use geo_bevy::{build_bevy_meshes, BuildBevyMeshesContext};
use gis_test::{read_geojson, read_geojson_feature_collection};
use proj::Transform;
use proj::Proj;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;

enum MeshType{
    Point,
    Polygon,
    LineString
}

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Default)]
struct SimpleOrthoProjection {
    near: f32,
    far: f32,
    aspect: f32,
}

impl CameraProjection for SimpleOrthoProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            -self.aspect, self.aspect, -1.0, 1.0, self.near, self.far
        )
    }

    // what to do on window resize
    fn update(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for SimpleOrthoProjection {
    fn default() -> Self {
        Self { near: 0.0, far: 1000.0, aspect: 1.0 }
    }
}

impl SimpleOrthoProjection{
    fn new_with_z(z: f32) -> Self{
        Self { near: 0.0, far: 1000.0, aspect: z}
    }
}


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
    app.add_plugin(bevy::render::camera::CameraProjectionPlugin::<SimpleOrthoProjection>::default());
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
    
    // events
    app.add_event::<gis_event::CenterCameraEvent>();
    app.add_event::<gis_event::CreateLayerEventReader>();
    app.add_event::<gis_event::CreateLayerEventWriter>();
    app.add_event::<gis_event::MeshSpawnedEvent>();
    app.add_event::<gis_event::PanCameraEvent>();
    app.add_event::<gis_event::PanCameraEvent>();
    app.add_event::<gis_event::ZoomCameraEvent>();
    
    // systems
    app.add_startup_system(setup);
    app.add_system(inspector_ui);
    //app.add_system(screen_to_world_dir);
    
    //resources
    //app.insert_resource(DefaultPluginState::<()>::default().with_debug_cursor());

    // run
    app.run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes_spawned_event_writer: bevy::ecs::event::EventWriter<MeshSpawnedEvent>
) {

    let mut layers = gis_layers::AllLayers::new();
    let feature_collection =
        read_geojson_feature_collection(read_geojson("maps/only_point.geojson".to_owned()));

    //proj instances
    let from = "EPSG:4326";
    let to = "EPSG:3875";
    
    let mut last_id = 0;
    let mut i = 0;

    for feature in feature_collection {
        let geometry = feature.geometry.unwrap();
        let geom: geo_types::geometry::Geometry<f64> = geometry.try_into().unwrap();

        // convert geom to proj format
        let mut new_geom:geo::Geometry = geom.clone().try_into().unwrap();
        let proj = Proj::new_known_crs(&from, &to, None).unwrap();
        new_geom.transform(&proj).unwrap();
        //let geom_projected = projection_geometry(new_geom.into(), proj);

        //println!("coord after projection: {:?}", geom_projected);

        let mesh_iter = build_bevy_meshes(&geom, Color::RED, BuildBevyMeshesContext::new())
            .unwrap()
            .collect::<Vec<_>>();
                        
        println!("coords {:?}",geom.clone());
        
        let _ = layers.add(geom, "mesh".to_owned(), to.to_owned());
        
        for prepared_mesh in mesh_iter {
            match prepared_mesh {
                geo_bevy::PreparedMesh::Point(points) => {
                     for geo::Point(coord) in points.iter() {
                        let color = Color::RED;
                        last_id = layers.last_layer_id();
                        let z_index = calculate_z(last_id, MeshType::Point) + i;
                        let mut transform =
                            bevy::prelude::Transform::from_xyz(coord.x as f32, coord.y as f32, z_index as f32);
                        transform.scale *= 0.7;

                        let bundle = SpriteBundle {
                            sprite: Sprite {
                                color,
                                ..Default::default()
                            },
                            texture: asset_server.load("circle.png"),
                            transform,
                            ..Default::default()
                        };

                        commands.spawn((bundle,RaycastMesh::<()>::default()));
                        // let meshes_spawned = MeshSpawnedEvent(gis_layer_id::new_id(last_id));
                        // meshes_spawned_event_writer.send(meshes_spawned);
                        i=i+1;
                    }
                },

                geo_bevy::PreparedMesh::Polygon { mesh, color } => {
                    let material = materials.add(color.into());
                    last_id = layers.last_layer_id();
                    let z_index = calculate_z(last_id, MeshType::Polygon);
                    let mut transform = bevy::prelude::Transform::from_xyz(0., 0., z_index  as f32);
                    transform.scale *= 0.7;
                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: transform,
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                    let meshes_spawned = MeshSpawnedEvent(gis_layer_id::new_id(last_id));
                    meshes_spawned_event_writer.send(meshes_spawned);
                },

                geo_bevy::PreparedMesh::LineString{mesh, color} => {
                    let material = materials.add(color.into());
                    last_id = layers.last_layer_id();
                    let z_index = calculate_z(last_id, MeshType::LineString);
                    println!("z_index for line string: {:?}",z_index);
                    let mut transform = bevy::prelude::Transform::from_xyz(0., 0., z_index  as f32);
                    transform.scale *= 0.7;
                    commands.spawn(MaterialMesh2dBundle {
                        material,
                        mesh: bevy::sprite::Mesh2dHandle(assets_meshes.add(mesh.clone())),
                        transform: transform,
                        visibility: bevy::render::view::Visibility { is_visible: true },
                        ..Default::default()
                    });
                    let meshes_spawned = MeshSpawnedEvent(gis_layer_id::new_id(last_id));
                    meshes_spawned_event_writer.send(meshes_spawned);
                },
            }
        }
    }

    // We need all the components that Bevy's built-in camera bundles would add
    // Refer to the Bevy source code to make sure you do it correctly:

    let projection = SimpleOrthoProjection::new_with_z(last_id as f32);

    // position the camera like bevy would do by default for 2D:
    let transform =  bevy::prelude::Transform::from_xyz(0.0, 0.0, projection.far);

    // frustum construction code copied from Bevy
    let view_projection =
        projection.get_projection_matrix() * transform.compute_matrix().inverse();
    let frustum = Frustum::from_view_projection(
        &view_projection,
        &transform.translation,
        &transform.back(),
        projection.far,
    );

    commands.spawn((
        bevy::render::camera::CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::NAME),
        projection,
        frustum,
        transform,
        GlobalTransform::default(),
        VisibleEntities::default(),
        bevy::render::camera::Camera::default(),
        bevy::core_pipeline::core_2d::Camera2d::default(),
        Tonemapping::Disabled,
    ));

}

// fn my_cursor_system(
//     mut mycoords: ResMut<MyWorldCoords>,
//     // query to get the window (so we can read the current cursor position)
//     mut q_window: ResMut<Windows>,
//     // query to get camera transform
//     q_camera: Query<(&bevy::render::camera::Camera, &GlobalTransform), With<MainCamera>>,
// ) {
//     // get the camera info and transform
//     // assuming there is exactly one main camera entity, so Query::single() is OK
//     let (camera, camera_transform) = q_camera.single();

//     // There is only one primary window, so we can similarly get it from the query:
//     let window = q_window.get_primary_mut().unwrap();

//     // check if the cursor is inside the window and get its position
//     // then, ask bevy to convert into world coordinates, and truncate to discard Z
//     if let Some(world_position) = window.cursor_position()
//         .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
//         .map(|ray| ray.origin.truncate())
//     {
//         mycoords.0 = world_position;
//         eprintln!("World coords: {}/{}", world_position.x, world_position.y);
//     }
// }

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

// fn setup_camera(mut commands: Commands) {
//     // We need all the components that Bevy's built-in camera bundles would add
//     // Refer to the Bevy source code to make sure you do it correctly:

//     // here we show a 2d example

//     let projection = SimpleOrthoProjection::default();

//     // position the camera like bevy would do by default for 2D:
//     let transform =  bevy::prelude::Transform::from_xyz(0.0, 0.0, projection.far - 0.1);

//     // frustum construction code copied from Bevy
//     let view_projection =
//         projection.get_projection_matrix() * transform.compute_matrix().inverse();
//     let frustum = Frustum::from_view_projection(
//         &view_projection,
//         &transform.translation,
//         &transform.back(),
//         projection.far,
//     );

//     commands.spawn((
//         bevy::render::camera::CameraRenderGraph::new(bevy::core_pipeline::core_2d::graph::NAME),
//         projection,
//         frustum,
//         transform,
//         GlobalTransform::default(),
//         VisibleEntities::default(),
//         bevy::render::camera::Camera::default(),
//         bevy::core_pipeline::core_2d::Camera2d::default(),
//         Tonemapping::Disabled,
//     ));
// }

// pub fn screen_to_world(
//     cursor: Res<bevy::ecs::event::Events<bevy::window::CursorMoved>>,
//     windows: Res<bevy::window::Windows>,
//     mut input_state: ResMut<GlobalInputState>,
//     camera_query: bevy::ecs::system::Query<(&bevy::render::camera::Camera, &dyn Transform)>,
// ) {
//     // Calculate world space mouse ray.  If there's no new mouse move event, then continue.
//     let pos = match input_state.cursor.latest(&cursor) {
//         Some(ev) => ev.position,
//         None => return,
//     };
//     let window = windows.get(bevy::window::WindowId::primary()).expect("Couldn't grab primary window.");
//     let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
//     let (c, c_t) = camera_query.iter().next().expect("The world didn't have a camera in it!");

//     let normal = screen_to_world_dir(&pos, &screen_size, c, c_t);
        
//     // This part can easily be done faster.
//     let camera_matrix = c_t.compute_matrix();
//     let (_, _, t) = camera_matrix.to_scale_rotation_translation();

//     let world_coord = t.translation + normal * DISTANCE;    // distance from the camera to put the world coord.
// }


// Convert a screen space coordinate to a direction out of the camera.
// Does not include camera positioning in the calculation: to get the screen to world coordinate,
// add camera_transform.translation here.
// pub fn screen_to_world_dir(
//     coord: &Vec2,
//     screen_size: &Vec2,
//     camera: &bevy::render::camera::Camera,
//     camera_transform: &dyn Transform<bevy::prelude::Camera2dBundle>
// ) -> Vec3 {
//     let proj_mat = camera.projection_matrix;

//     // Normalized device coordinates (NDC) describes cursor position from (-1, -1) to (1, 1).
//     let cursor_pos_ndc: Vec3 = ((*coord / *screen_size) * 2.0 - Vec2::one()).extend(1.0);

//     let camera_matrix = camera_transform.compute_matrix();
//     // We can't just use camera_transform.translation because that's actually the LOCAL translation.
//     let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();
    
//     let ndc_to_world: bevy::math::Mat4 = camera_matrix * proj_mat.inverse();
//     let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);
    
//     let dir: Vec3 = cursor_position - camera_position;

//     return dir.normalize();
// }

// fn projection_geometry(geom: geo_types::Geometry, proj: Proj) -> geo_types::Geometry {
//     match geom {
//         geo_types::Geometry::Point(point) => {
//             let new_point = proj.convert(point).unwrap();
//             geo_types::Geometry::Point(new_point)
//         }
//         geo_types::Geometry::Polygon(polygon) => {
//             let proj_poly = polygon.transformed(&proj).unwrap();
//             geo_types::Geometry::Polygon(proj_poly)
//         }
//         geo_types::Geometry::LineString(line) => {
//             let new_line = line.transformed(&proj).unwrap();
//             geo_types::Geometry::LineString(new_line)
//         }
//         _ => unimplemented!(),
//     }
// }