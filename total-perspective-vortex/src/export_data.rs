use std::fs;
use std::path::Path;

use crate::export_types::*;

pub fn generate_preparation_movement(start_position: (f32, f32, f32)) -> DeltaAction {
    let home_at_position = vec![start_position];

    let prep_move = DeltaAction {
        id: 1,
        action: String::from("queue_movement"),
        payload: Motion {
            id: 1, // homing makes sense as first ID
            reference: 0,
            motion_type: 0,
            duration: 2500,
            points: home_at_position,
        },
    };

    return prep_move;
}

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

pub fn export_uv(write_path: &Path, data: f32) {
    println!("TODO implement UV exporter");
}
