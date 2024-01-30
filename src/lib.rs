mod gis_layer_id;
mod gis_layers;

use bevy::math::{Vec2, Vec3};
use bevy::transform::components::Transform;
use bevy_prototype_lyon::geometry::*;
use geo::{CoordsIter, GeometryCollection};
use geo_types::Point;
use geojson::{quick_collection, FeatureCollection, GeoJson};
use std::fs;
use std::str::FromStr;

pub enum MeshType {
    Point,
    Polygon,
    LineString,
}

pub fn read_geojson(path: String) -> GeoJson {
    let geojson_str = fs::read_to_string(path).unwrap();
    GeoJson::from_str(&geojson_str).unwrap()
}

//read geometry with a feature collection
pub fn read_geojson_feature_collection(geojson: GeoJson) -> FeatureCollection {
    let collection: GeometryCollection = quick_collection(&geojson).unwrap();
    FeatureCollection::from(&collection)
}

pub fn calculate_z(layer_index: i32, mesh_type: MeshType) -> f32 {
    return layer_index as f32 * 3.
        + match mesh_type {
            MeshType::Point => 1.,
            MeshType::Polygon => 2.,
            MeshType::LineString => 3.,
        };
}

pub fn medium_centroid(centroids: Vec<Point>) -> Point {
    let mut somma_x = 0.0;
    let mut somma_y = 0.0;

    for centroid in centroids.clone() {
        somma_x += centroid.0.x;
        somma_y += centroid.0.y;
    }

    Point::new(
        somma_x / centroids.len() as f64,
        somma_y / centroids.len() as f64,
    )
}

pub fn build_polygon(
    polygon: geo_types::geometry::Polygon,
    id: i32,
) -> (GeometryBuilder, Transform) {
    let mut coords: Vec<Vec2> = Vec::new();

    for coord in polygon.coords_iter() {
        coords.push(Vec2 {
            x: coord.x as f32,
            y: coord.y as f32,
        });
    }

    let shape = bevy_prototype_lyon::shapes::Polygon {
        points: coords,
        closed: true,
    };
    let z = calculate_z(id, MeshType::Polygon);
    let translation = Vec3 { x: 0., y: 0., z };
    let transform = Transform::from_translation(translation);

    (GeometryBuilder::new().add(&shape), transform)
}

pub fn build_linestring(
    line_string: geo_types::geometry::LineString,
    id: i32,
) -> (GeometryBuilder, Transform) {
    let mut coords: Vec<Point> = Vec::new();

    for coord in line_string.0 {
        coords.push(Point::new(coord.x as f64, coord.y as f64));
    }

    let start = coords.get(0).unwrap();
    let last = coords.last().unwrap();

    let shape = bevy_prototype_lyon::shapes::Line(
        Vec2 {
            x: start.0.x as f32,
            y: start.0.y as f32,
        },
        Vec2 {
            x: last.0.x as f32,
            y: last.0.y as f32,
        },
    );

    let builder = GeometryBuilder::new().add(&shape);
    let translation = Vec3 {
        x: 0.,
        y: 0.,
        z: calculate_z(id, MeshType::LineString),
    };
    let transform = Transform::from_translation(translation);

    (builder,transform)
}
