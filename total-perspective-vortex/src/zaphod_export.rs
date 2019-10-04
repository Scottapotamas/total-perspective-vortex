use std::fs;
use std::path::Path;

use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct DeltaEvents {
    pub metadata: EventMetadata,
    pub actions: Vec<ActionGroups>,
}

#[derive(Serialize, Debug)]
pub struct EventMetadata {
    pub name: String,
    #[serde(rename = "formatVersion")]
    pub format_version: String,
}

#[derive(Serialize, Debug)]
pub struct ActionGroups {
    pub delta: Vec<DeltaAction>,
    pub light: Vec<LightAction>,
    pub run: Vec<GenericAction>,
}

#[derive(Serialize, Debug)]
pub struct DeltaAction {
    pub id: u32,
    pub action: String,
    pub payload: Motion,
    //    #[serde(skip_serializing_if = "is_null")]
    //    waitFor: u32,
}

#[derive(Serialize, Debug)]
pub struct Motion {
    #[serde(rename = "type")]
    pub motion_type: u32,
    pub reference: u32,
    pub id: u32,
    pub duration: u32,
    pub points: Vec<(f32, f32, f32)>,
}

#[derive(Serialize, Debug)]
pub struct LightAction {
    pub id: u32,
    pub action: String,
    pub payload: LightAnimation,
    pub comment: String,
    //    #[serde(skip_serializing_if = "is_null")]
    //    waitFor: u32,
}

#[derive(Serialize, Debug)]
pub struct LightAnimation {
    #[serde(rename = "type")]
    pub animation_type: u32,
    pub id: u32,
    pub duration: u32,
    pub points: Vec<(f32, f32, f32)>,
}

#[derive(Serialize, Debug)]
pub struct GenericAction {
    pub id: u32,
    pub action: String,
    pub payload: String,
    pub comment: String,
    //    #[serde(skip_serializing_if = "is_null")]
    #[serde(rename = "waitFor")]
    pub wait_for: u32,
}

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
