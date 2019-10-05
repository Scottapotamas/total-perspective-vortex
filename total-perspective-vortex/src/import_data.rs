use std::fs::File;
use std::path::Path;

extern crate image;
use self::image::DynamicImage;
use image::{imageops, GenericImageView};

extern crate colorsys;
use colorsys::{Hsl, Rgb};

use crate::delta_utils::*;
use crate::import_types::*;

// Parses the JSON spline data generated by the Blender python export script
// Finds the UV map referenced in the JSON file, scrapes image data
// Returns the spline and illumination data
pub fn load_blender_data(input_path: &Path) -> IlluminatedSpline {
    let folder_root = input_path
        .parent()
        .expect("Error getting parent path of JSON");

    let json_file = File::open(input_path).expect("JSON file not found");

    let mut bl_spline: BlenderSpline =
        serde_json::from_reader(json_file).expect("Error parsing json");

    let uv_relative_path = Path::new(&bl_spline.uv_path);
    let uv_full_path = folder_root.join(&uv_relative_path);

    let input_colors = match load_uv(uv_full_path.as_path()) {
        Ok(contents) => convert_uv(contents),
        Err(_error) => generate_placeholder_uv_data(),
    };

    // Apply coordinate transforms like scaling/offsets
    transform_meters_to_millimeters(&mut bl_spline.points);
    transform_z_axis(&mut bl_spline.points, 30.0);
    bl_spline.curve_length *= 100.0;

    if bl_spline.cyclic {
        // Duplicate the first point(s) into the tail to close the loop
        match bl_spline.spline_type.as_str() {
            "poly" => {
                // Put the first point at the end of the set
                bl_spline
                    .points
                    .push(bl_spline.points.first().unwrap().clone());
            }
            "nurbs" => {
                // Put the first two points at the end of the set
                bl_spline.points.push(bl_spline.points[0].clone());
                bl_spline.points.push(bl_spline.points[1].clone());
                bl_spline.points.push(bl_spline.points[2].clone());
            }
            _ => {
                println!("Unsupported blender data type");
            }
        }
    }

    // Perform LED manipulation here
    // TODO consider supporting color inversion?

    return IlluminatedSpline {
        spline: bl_spline,
        illumination: input_colors,
    };
}

fn load_uv(input_path: &Path) -> Result<DynamicImage, image::ImageError> {
    let img = image::open(input_path)?;
    Ok(img)
}

// The blender exported UV map is a X*Y sized RGB8 PNG file and we want a 1D set of HSL colours
fn convert_uv(image: DynamicImage) -> Vec<Hsl> {
    let width = image.dimensions().0;

    let mut next_img = image.clone();
    let first_row = imageops::crop(&mut next_img, 0, 0, width, 1);

    let hue_list: Vec<Hsl> = first_row
        .pixels()
        .into_iter()
        .map(|pixel| {
            Rgb::from((
                pixel.2[0] as f64,
                pixel.2[1] as f64,
                pixel.2[2] as f64,
                pixel.2[3] as f64,
            ))
            .as_ref()
            .into()
        })
        .collect();

    return hue_list;
}

// Create a fallback white fade pair to provide lighting on moves which didn't have a valid UV map provided.
fn generate_placeholder_uv_data() -> Vec<Hsl> {
    return vec![Hsl::new(0.0, 0.0, 50.0, Option::from(1.0)); 2];
}
