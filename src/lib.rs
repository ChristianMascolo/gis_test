use geo::GeometryCollection;
use geojson::{quick_collection, FeatureCollection, GeoJson};
use std::fs;
use std::str::FromStr;

pub fn read_geojson(path: String) -> GeoJson {
    let geojson_str = fs::read_to_string(path).unwrap();
    GeoJson::from_str(&geojson_str).unwrap()
}

//read geometry with a feature collection
pub fn read_geojson_feature_collection(geojson: GeoJson) -> FeatureCollection {
    let collection: GeometryCollection = quick_collection(&geojson).unwrap();
    FeatureCollection::from(&collection)
}
