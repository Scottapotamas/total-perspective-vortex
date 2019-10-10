use std::fs;
use std::path::Path;

use crate::color_utils::*;
use crate::export_types::*;
use colorsys::Hsl;

use image::imageops::resize;
use image::{FilterType, ImageBuffer, Rgb, RgbImage};

pub fn generate_header(title: String) -> EventMetadata {
    return EventMetadata {
        format_version: String::from("0.0.1"),
        name: title,
    };
}

pub fn export_toolpath(write_path: &Path, data: DeltaEvents) {
    let data_to_write = serde_json::to_string_pretty(&data).expect("Serialisation Failed");
    fs::write(write_path, data_to_write).expect("Unable to write file");
}

pub fn export_vertices(write_path: &Path, data: Vec<(f32, f32, f32)>) {
    let data_to_write = serde_json::to_string_pretty(&data).expect("Serialisation Failed");
    fs::write(write_path, data_to_write).expect("Unable to write file");
}

pub fn export_uv(write_path: &Path, data: Vec<Hsl>) {
    let mut image_buffer: RgbImage = ImageBuffer::new(data.len() as u32, 16);

    for (x, _y, pixel) in image_buffer.enumerate_pixels_mut() {
        let hue = data[x as usize].clone();
        let (red, green, blue): (u8, u8, u8) = hsl_to_rgb8(&hue);

        *pixel = Rgb([red, green, blue]);
    }

    let img_dims = image_buffer.dimensions();

    let new_size = resize(
        &image_buffer,
        next_power_two(img_dims.0),
        16,
        FilterType::Gaussian,
    );

    new_size.save(write_path).unwrap();
}

fn next_power_two(input: u32) -> u32 {
    for x in 3..13 {
        let p2 = 2_u32.pow(x);
        if input < p2 {
            return p2;
        }
    }
    return 4096;
}
