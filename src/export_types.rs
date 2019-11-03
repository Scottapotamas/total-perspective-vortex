use serde::Serialize;
use serde_repr::{Serialize_repr};

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

    #[serde(skip_serializing)]
    global_id: u32, // all moves, lights, extra actions need a unique global ID, as json doesn't guarantee order

    #[serde(skip_serializing)]
    move_time: u32,
}

pub trait Actions {
    fn new() -> ActionGroups;

    fn add_delta_action(&mut self, m: Motion);
    fn add_light_action(&mut self, l: Fade);
    fn add_generic_action(&mut self, a: String, p: String);

    fn get_next_global_id(&self) -> u32;
    fn get_movement_duration(&self) -> u32;
}

impl Actions for ActionGroups {
    fn new() -> ActionGroups {
        ActionGroups {
            delta: vec![],
            light: vec![],
            run: vec![],
            global_id: 0,
            move_time: 0,
        }
    }

    fn add_delta_action(&mut self, mut m: Motion) {
        // Set the ID for the move being added to the set
        m.id = self.delta.len() as u32 + 1;

        // Accumulate movement time (don't count transit moves)
        if m.motion_type != MotionInterpolationType::PointTransit {
            self.move_time += m.duration;
        }

        self.delta.push(DeltaAction {
            id: self.global_id,
            action: String::from("queue_movement"),
            payload: m,
        });

        self.global_id += 1;
    }

    fn add_light_action(&mut self, mut l: Fade) {
        l.id = self.light.len() as u32 + 1;

        self.light.push(LightAction {
            id: self.global_id,
            action: "queue_light".to_string(),
            payload: l,
            comment: "".to_string(),
        });

        self.global_id += 1;
    }

    fn add_generic_action(&mut self, a: String, p: String) {
        self.run.push(GenericAction {
            id: self.global_id,
            action: a,
            payload: p,
            comment: "".to_string(),
            wait_for: 0,
        });

        self.global_id += 1;
    }

    fn get_next_global_id(&self) -> u32 {
        self.global_id
    }

    fn get_movement_duration(&self) -> u32 {
        self.move_time
    }
}

#[derive(Serialize, Debug)]
pub struct DeltaAction {
    pub id: u32,
    pub action: String,
    pub payload: Motion,
}

#[derive(Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum MotionInterpolationType {
    PointTransit = 0,
    Line = 1,
    CatmullSpline = 2,
    BezierQuadratic = 3,
    BezierCubic = 4,
}

#[derive(Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum MotionReferenceFrame {
    Absolute = 0,
    Relative = 1,
}

#[derive(Serialize, Debug)]
pub struct Motion {
    #[serde(rename = "type")]
    pub motion_type: MotionInterpolationType,
    pub reference: MotionReferenceFrame,
    pub id: u32,
    pub duration: u32,
    pub points: Vec<(f32, f32, f32)>,
}

#[derive(Serialize, Debug)]
pub struct LightAction {
    pub id: u32,
    pub action: String,
    pub payload: Fade,
    pub comment: String,
}

#[derive(Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum LightAnimationType {
    ConstantOn = 0,
    LinearFade = 1,
}

#[derive(Serialize, Debug)]
pub struct Fade {
    #[serde(rename = "type")]
    pub animation_type: LightAnimationType,
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
