use std::fs::File;
use std::path::Path;

use serde::Deserialize;

extern crate image;
use image::{imageops, GenericImageView};

extern crate colorsys;
use self::image::DynamicImage;
use colorsys::{Hsl, Rgb};

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

// Parses the JSON spline data generated by the Blender python export script
// Finds the UV map referenced in the JSON file, scrapes image data
// Returns the spline and illumination data
pub fn load_blender_data(input_path: &Path) -> IlluminatedSpline {
    let folder_root = input_path
        .parent()
        .expect("Error getting parent path of JSON");

    let json_file = File::open(input_path).expect("JSON file not found");

    let bl_spline: BlenderSpline = serde_json::from_reader(json_file).expect("Error parsing json");

    let uv_relative_path = Path::new(&bl_spline.uv_path);
    let uv_full_path = folder_root.join(&uv_relative_path);

    let input_colors = match load_uv(uv_full_path.as_path()) {
        Ok(contents) => convert_uv(contents),
        Err(_error) => generate_placeholder_uv_data(),
    };

    let temp: IlluminatedSpline = IlluminatedSpline {
        spline: bl_spline,
        illumination: input_colors,
    };

    return temp;
}

// Convert the blender co-ordinate units to millimeters
pub fn transform_meters_to_millimeters(points: &mut Vec<BlenderPoint>) {
    for point in points {
        point.x *= 100.0;
        point.y *= 100.0;
        point.z *= 100.0;

        point.z += 30.0;
    }
}

// Calculate the 3D distance in mm between two points
fn distance_3d(a: &BlenderPoint, b: &BlenderPoint) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    let distance = ((dx * dx) + (dy * dy) + (dz * dz)).sqrt();

    return distance.abs();
}

fn interpolate_catmull_point(p: &[BlenderPoint], weight: f32) -> Result<BlenderPoint, String> {
    if weight <= 0.0 || weight >= 1.0 {
        // Weights should be between 0.0-1.0 representing the percentage point to interpolate
        return Err("Can't interpolate catmull with input weight".to_string());
    }

    let t = weight;
    let t2 = t * t;
    let t3 = t2 * t;

    /* Derivation from http://www.mvps.org/directx/articles/catmull/

                                [  0  2  0  0 ]   [ p0 ]
    q(t) = 0.5( t, t^2, t^3 ) * [ -1  0  1  0 ] * [ p1 ]
                                [  2 -5  4 -1 ]   [ p2 ]
                                [ -1  3 -3  1 ]   [ p3 ]
     */

    let out_x = 0.5
        * ((2.0 * p[1].x)
            + (-p[0].x + p[2].x) * t
            + (2.0 * p[0].x - 5.0 * p[1].x + 4.0 * p[2].x - p[3].x) * t2
            + (-p[0].x + 3.0 * p[1].x - 3.0 * p[2].x + p[3].x) * t3);

    let out_y = 0.5
        * ((2.0 * p[1].y)
            + (-p[0].y + p[2].y) * t
            + (2.0 * p[0].y - 5.0 * p[1].y + 4.0 * p[2].y - p[3].y) * t2
            + (-p[0].y + 3.0 * p[1].y - 3.0 * p[2].y + p[3].y) * t3);

    let out_z = 0.5
        * ((2.0 * p[1].z)
            + (-p[0].z + p[2].z) * t
            + (2.0 * p[0].z - 5.0 * p[1].z + 4.0 * p[2].z - p[3].z) * t2
            + (-p[0].z + 3.0 * p[1].z - 3.0 * p[2].z + p[3].z) * t3);

    Ok(BlenderPoint {
        x: out_x,
        y: out_y,
        z: out_z,
        w: 0.0,
    })
}

// Estimate the 3D length of a catmull-rom spline by sampling repeatedly
fn distance_catmull(control_points: &[BlenderPoint]) -> Result<f32, String> {
    let mut accumulated_length: f32 = 0.0;

    let samples: Vec<u32> = (1..99).collect();

    for test_point in samples.windows(2) {
        let a = interpolate_catmull_point(control_points, test_point[0] as f32 * 0.01)?;
        let b = interpolate_catmull_point(control_points, test_point[1] as f32 * 0.01)?;

        accumulated_length += distance_3d(&a, &b);
    }

    Ok(accumulated_length)
}

pub fn calculate_duration(points: &[BlenderPoint], speed: f32) -> Result<f32, String> {
    let mut distance = 0.0;

    match points.len() {
        1 => return Err("Duration for one point?".to_string()),
        2 => {
            distance = distance_3d(&points[0], &points[1]);
        }
        4 => {
            distance = distance_catmull(points)?;
        }
        _ => return Err("Can't calculate duration on this number of points".to_string()),
    }

    let duration = distance / speed;
    //    println!(
    //        "Effector will take {} seconds to travel {} mm at {} mm/sec",
    //        duration, distance, speed
    //    );

    Ok(duration)
}

fn load_uv(input_path: &Path) -> Result<DynamicImage, image::ImageError> {
    let img = image::open(input_path)?;
    Ok(img)
}

// The blender exported UV map is a X*Y sized RGB8 PNG file and we want a 1D set of HSL colours
fn convert_uv(image: DynamicImage) -> Vec<Hsl> {
    let width = image.dimensions().0;
    //    println!("UV Map is {}px long, in {:?}", width, image.color());

    let mut next_img = image.clone();
    let first_row = imageops::crop(&mut next_img, 0, 0, width, 1);

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

// Create a fallback white 2-point value pair to provide lighting on moves which didn't have a valid UV map provided.
fn generate_placeholder_uv_data() -> Vec<Hsl> {
    //    println!("Generating white colour as fallback for missing UV");

    let mut hue_list: Vec<Hsl> = Vec::new();
    hue_list.push(Hsl::new(0.0, 0.0, 0.5, Option::from(1.0)));
    hue_list.push(Hsl::new(0.0, 0.0, 0.5, Option::from(1.0)));

    return hue_list;
}

// Calculates the distance between two HSL values as expected by human vision models
// Its OK to add a constant k to the lightness difference as it has arbitrary importance
pub fn distance_hsl(x: &Hsl, y: &Hsl) -> f64 {
    // HSL must be [0, 2pi), [0, 1], [0, 1] first
    let h1 = x.get_hue().to_radians();
    let s1 = x.get_saturation();
    let l1 = x.get_lightness();

    let h2 = y.get_hue().to_radians();
    let s2 = y.get_saturation();
    let l2 = y.get_lightness();

    // Project the colors into linear space (a,b,c)
    let a1 = h1.cos() * s1 * l1;
    let b1 = h1.sin() * s1 * l1;
    let c1 = l1;

    let a2 = h2.cos() * s2 * l2;
    let b2 = h2.sin() * s2 * l2;
    let c2 = l2;

    // Simple cartesian distance
    let d2 = (a1 - a2).powi(2) + (b1 - b2).powi(2) + (c1 - c2).powi(2);
    return d2.sqrt();
}
