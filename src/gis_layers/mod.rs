use bevy_asset::Error;
use crate::gis_layer_id::*;

pub struct Layer{
    pub id: LayerId,
    pub name: String,
    pub crs: String,
    pub geom_type: geo_types::Geometry,
    pub visible: bool,
}

pub struct AllLayers{
    layers: Vec<Layer>,
    pub selected_layer_id: i32,
}

impl AllLayers{
    pub fn new() -> AllLayers{
        AllLayers{
            layers: vec![],
            selected_layer_id: 0,
        }
    }

    pub fn iter_bottom_to_top(&self) -> impl Iterator<Item = &Layer>{
        self.layers.iter()
    }

    pub fn iter_top_to_bottom(&self) -> impl Iterator<Item = &Layer>{
            self.layers.iter().rev()
    }

    pub fn count(&self) -> usize {
        self.layers.len()
    }

    fn next_layer_id(&self) -> LayerId {
        LayerId::new()
    }

    pub fn add(
        &mut self,
        geometry: geo_types::Geometry,
        name: String,
        crs: String,
    ) -> Result<LayerId, Error>{
        let id = self.next_layer_id();
        let layer = Layer{
            id,
            name,
            crs,
            visible: false,
            geom_type: geometry,
        };

        self.layers.push(layer);
        Ok(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    pub fn last_layer_id(&self) -> i32{
        self.layers.last().unwrap().id.get_id()
    }
}