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
    pub duration: f32,
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
