use std::path::{Path, PathBuf};

extern crate walkdir;
use walkdir::{DirEntry, WalkDir};

pub mod import_data;
pub mod import_types;
use import_data::*;
use import_types::*;

pub mod export_data;
pub mod export_types;
use export_data::*;
use export_types::*;

pub mod sequencer;
use sequencer::*;

pub mod color_utils;
pub mod delta_utils;

use itertools::Itertools;
use serde::Serialize;
use std::fs;

// Checks that a DirEntry isn't hidden, a __MACOSX folder, or a file
fn is_frame_folder(entry: &DirEntry) -> bool {
    if entry.file_type().is_dir() {
        entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with('.') && !s.starts_with("__"))
            .unwrap_or(false)
    } else {
        false
    }
}

fn is_json_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".json")) //&& !s.ends_with("toolpath.json")
        .unwrap_or(false)
}

fn main() {
    println!("Welcome to the Total Perspective Vortex!");

    // Walk the folder structure looking for frame folders, then process them
    let mut frame_meta: Vec<FrameMetadata> = WalkDir::new("./")
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_frame_folder(e))
        .filter_map(|v| v.ok())
        .map(|x| process_frame_folder(&x))
        .collect();

    // Sort the frames in ascending numeric order. Unstable sort means equal elements may move
    frame_meta.sort_unstable_by_key(|x| x.frame_num);

    let mut all_collection_names = vec![];

    for frame in &frame_meta {
        for collection in &frame.collections {
            all_collection_names.push(collection.name.clone());
        }
    }
    let unique_collection_names: Vec<String> = all_collection_names.into_iter().unique().collect();

    let summary = AnimationMetadata {
        collections: unique_collection_names,
        frames: frame_meta,
    };

    let overview_file =
        serde_json::to_string_pretty(&summary).expect("Summary Serialisation Failed");
    fs::write(Path::new("./summary.json"), overview_file).expect("Unable to write file");
}

// From a valid frame folder, find collections folders to process
fn process_frame_folder(entry: &DirEntry) -> FrameMetadata {
    let frame_folder_name = entry.file_name().to_string_lossy();
    let frame_number = frame_folder_name.parse::<i32>().unwrap();
    println!("\nProcessing Frame {}", frame_folder_name);

    let exported_file_metadata: Vec<FileMetadata> = WalkDir::new(entry.path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir())
        .filter_map(|v| v.ok())
        .map(|x| process_collection(&x))
        .filter(|y| y.is_some())
        .map(|z| z.unwrap())
        .collect();

    FrameMetadata {
        frame_num: frame_number,
        collections: exported_file_metadata,
    }
}

#[derive(Serialize, Debug)]
struct AnimationMetadata {
    collections: Vec<String>,
    frames: Vec<FrameMetadata>,
}

#[derive(Serialize, Debug)]
struct FrameMetadata {
    frame_num: i32,
    collections: Vec<FileMetadata>,
}

#[derive(Serialize, Debug)]
struct FileMetadata {
    name: String,
    toolpath_path: String,
    duration: u32,
    first_move: u32,
    last_move: u32,
    num_lights: u32,
    viewer_vertices_path: String,
    viewer_uv_path: String,
}

// A collection is the deepest level folder. Contains json and (optional) uv files from Blender
fn process_collection(entry: &DirEntry) -> Option<FileMetadata> {
    // Parse all the json files in the current directory
    let parsed_splines: Vec<BlenderData> = WalkDir::new(entry.path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_json_file(e))
        .filter_map(|v| v.ok())
        .map(|x| load_blender_data(&x.path()))
        .collect();

    if parsed_splines.is_empty()
    {
        return None;
    }

    // Take our spline+illumination data, and generate a tool-path
    let planned_events = generate_delta_toolpath(&parsed_splines);

    if planned_events.delta.is_empty()
    {
        return None;
    }

    // Generate additional exports for use in the UI as previz data
    let viewer_preview = generate_viewer_data(&parsed_splines);

    let duration: u32 = planned_events
        .delta
        .iter()
        .map(|x| x.payload.duration)
        .sum();

    let first_move = planned_events.delta.first().expect("No first move").payload.id;
    let last_move = planned_events.delta.last().expect("No last move").payload.id;

    let num_lights = planned_events.light.len() as u32;
    // Add header information
    let output_data: DeltaEvents = DeltaEvents {
        metadata: generate_header(String::from("VortexFile")),
        actions: vec![planned_events],
    };

    // Put the output JSON in the parent folder alongside the other collection exports
    let collection_name = entry
        .path()
        .file_name()
        .expect("Couldn't get collection name")
        .to_str()
        .expect("Failed converting collection name to string")
        .to_string();

    let destination_folder = entry.path().parent().unwrap();

    let delta_path = format_filename(
        destination_folder,
        collection_name.clone(),
        "toolpath.json".to_string(),
    );

    let vertex_path = format_filename(
        destination_folder,
        collection_name.clone(),
        "vertices.json".to_string(),
    );

    let uv_path = format_filename(
        destination_folder,
        collection_name.clone(),
        "uv.png".to_string(),
    );

    // Write to disk
    export_toolpath(&delta_path.as_path(), output_data);
    export_vertices(&vertex_path.as_path(), viewer_preview.0);
    export_uv(&uv_path.as_path(), viewer_preview.1);

    Some(FileMetadata {
        name: collection_name,
        toolpath_path: pathbuf_to_string(delta_path),
        duration,
        first_move,
        last_move,
        num_lights,
        viewer_vertices_path: pathbuf_to_string(vertex_path),
        viewer_uv_path: pathbuf_to_string(uv_path),
    })
}

fn pathbuf_to_string(input: PathBuf) -> String {
    return input.to_str().unwrap().to_string();
}

// Takes a destination folder, the name of the collection, and the extension of the file
// Returns a path to the location of the file, with a cleaner filename
fn format_filename(destination: &Path, name: String, extension: String) -> PathBuf {
    let mut collection_name = name.to_lowercase();
    collection_name.retain(|c| !c.is_whitespace());

    let file_name = format!("{}_{}", collection_name, extension);
    let path = Path::new(&file_name);
    let parent_folder = destination.clone();

    parent_folder.join(&path)
}
