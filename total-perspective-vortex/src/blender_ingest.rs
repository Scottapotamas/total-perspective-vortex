use std::fs::File;
use std::path::Path;

use serde::Deserialize;

extern crate image;
use image::{imageops, GenericImageView};

extern crate colorsys;
use colorsys::{Hsl, Rgb};

#[derive(Deserialize, Debug)]
pub struct BlenderSpline {
    pub curve_length: f32,
    pub points: Vec<SplinePoint>,
}

#[derive(Deserialize, Debug)]
pub struct SplinePoint {
    #[serde(rename = "co")]
    pub point: Vec<f32>,
}

// Parses the JSON spline data generated by the Blender python export script
pub fn load_json(input_path: &Path) -> BlenderSpline {
    let json_file_path = Path::new(input_path);

    let json_file = File::open(json_file_path).expect("JSON file not found");

    let bl_spline: BlenderSpline =
        serde_json::from_reader(json_file).expect("Error while reading json");

    println!("Length is: {}", bl_spline.curve_length);

    return bl_spline;
}

// Loads the UV export image from blender, and returns a vector of HSV colours
pub fn load_uv(input_path: &Path) -> Vec<Hsl> {
    let mut img = image::open(input_path).unwrap();
    let width = img.dimensions().0;
    println!("UV Map is {}px long, in {:?}", width, img.color());

    let first_row = imageops::crop(&mut img, 0, 0, width, 1);

    let mut hue_list: Vec<Hsl> = Vec::new();

    for pixel in first_row.pixels() {
        let rbga_tuple = (
            pixel.2[0] as f64,
            pixel.2[1] as f64,
            pixel.2[2] as f64,
            pixel.2[3] as f64,
        );
        let rgba = Rgb::from(&rbga_tuple);
        let hsla: Hsl = rgba.as_ref().into();

        hue_list.push(hsla);
    }

    return hue_list;
}