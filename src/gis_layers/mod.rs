use bevy_asset::Error;
use crate::gis_layer_id::*;

pub struct Layer{
    pub id: LayerId,
    pub name: String,
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

    fn next_layer_id(&self) -> LayerId {
        LayerId::new(self.last_layer_id())
    }

    pub fn add(
        &mut self,
        geometry: geo_types::Geometry,
        name: String,
    ) -> Result<LayerId, Error>{
        let id = self.next_layer_id();
        let layer = Layer{
            id,
            name,
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
        if self.layers.len() == 0 { return 0; }

        self.layers.last().unwrap().id.get_id()
    }
}