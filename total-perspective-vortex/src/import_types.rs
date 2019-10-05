use colorsys::Hsl;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct BlenderSpline {
    pub curve_length: f32,
    pub points: Vec<BlenderPoint>,
    #[serde(rename = "type")]
    pub spline_type: String,
    #[serde(rename = "uv")]
    pub uv_path: String,
    #[serde(default)]
    pub cyclic: bool,
    #[serde(skip)]
    pub target_duration: f32,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct BlenderPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub struct IlluminatedSpline {
    pub spline: BlenderSpline,
    pub illumination: Vec<Hsl>,
}
