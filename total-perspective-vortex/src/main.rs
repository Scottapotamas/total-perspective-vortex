use std::path::Path;

extern crate walkdir;
use walkdir::{DirEntry, WalkDir};

pub mod blender_ingest;
use blender_ingest::*;

pub mod zaphod_export;
use zaphod_export::*;

pub mod sequencer;
use sequencer::*;

// Checks that a DirEntry isn't hidden, a __MACOSX folder, or a file
fn is_frame_folder(entry: &DirEntry) -> bool {
    if entry.file_type().is_dir() {
        entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with(".") && !s.starts_with("__"))
            .unwrap_or(false)
    } else {
        return false;
    }
}

fn main() {
    println!("Welcome to the Total Perspective Vortex!");

    // Walk the folder structure looking for frame folders, then process them
    WalkDir::new("./multi")
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_frame_folder(e))
        .filter_map(|v| v.ok())
        .for_each(|x| process_frame_folder(&x));
}

fn is_json_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".json") && !s.ends_with("toolpath.json"))
        .unwrap_or(false)
}

// From a valid frame folder, find files, process, and generate the tool-path
fn process_frame_folder(entry: &DirEntry) {
    println!("\nProcessing Frame {:?}", entry.file_name());

    // Parse the json files in the current directory
    let mut parsed_splines: Vec<IlluminatedSpline> = WalkDir::new(entry.path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_json_file(e))
        .filter_map(|v| v.ok())
        .map(|x| load_blender_data(&x.path()))
        .collect();

    // Manipulate parsed data before any planning is done
    for glowy_spline in &mut parsed_splines {
        // Apply transforms like scaling/offsets
        transform_meters_to_millimeters(&mut glowy_spline.spline.points);
        glowy_spline.spline.curve_length *= 100.0;

        if glowy_spline.spline.cyclic {
            // Duplicate the first point(s) into the tail to close the loop
            // todo support closed splines

        }

        // Perform any LED manipulation here
        // TODO consider supporting color inversion?
    }

    /*
    let min_duration: f32 = parsed_splines
        .iter()
        .map(|s| s.spline.target_duration)
        .sum();
    println!("Total time is {}", min_duration);
    */

    // Take our spline+illumination data, and generate an optimised tool-path
    let planned_events = sequence_events(parsed_splines);

    //    let render_duration: f32 = parsed_splines
    //        .iter()
    //        .map(|s| s.spline.target_duration)
    //        .sum();
    //    println!("Total time is {}", render_duration);

    // Add header information
    let output_data: DeltaEvents = DeltaEvents {
        metadata: generate_header(String::from("VortexFile")),
        actions: vec![planned_events],
    };

    // Put the output JSON in the folder alongside the input files
    let export_json_name = Path::new("toolpath.json");
    let destination_path = entry.path().join(&export_json_name);

    // Write to the JSON file in format suitable for zaphod-bot
    export_toolpath(destination_path.as_path(), output_data);
}
