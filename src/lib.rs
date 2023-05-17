use geo::GeometryCollection;
use geojson::{quick_collection, FeatureCollection, GeoJson, Geometry, Feature};
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

//read geometry with only Feature
#[allow(dead_code)]
pub fn read_geojson_feature(geojson: GeoJson)->Geometry {
    let feature: Feature = Feature::try_from(geojson).unwrap();

    feature.geometry.unwrap()
}