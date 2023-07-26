use bevy::prelude::*;

fn created_layer_event(
    mut create_layer_events: ResMut<bevy::ecs::event::Events<gis_events::CreateLayerReader>>,
    mut layer_created_event_writer: EventWriter<gis_test::CreateLayerWriter>,
    mut layers: ResMut<crate::AllLayers>,
){
    for event in create_layer_events.drain(){
        match layers.add(event.name,event.crs,event.coord) {
            Ok(layer_id) => {
                layer_created_event_writer.send(gis_event::CreateLayerWriter(layer_id))
            }
            Err(e) => bevy::log::error!("Encountered error when creating layer: {:?}", e),
        }
    }
}

pub fn configure(app: &mut App){
    add.add_system(created_layer_event);
}