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
    delta: Vec<DeltaAction>,
    light: Vec<LightAction>,
    run: Vec<GenericAction>,

    #[serde(skip_serializing)]
    global_id: u32,  // all moves, lights, extra actions need a unique global ID, as json doesn't guarantee order
}

pub trait Actions {
    fn add_delta_action(&mut self, mut m: Motion );
    fn add_light_action(&mut self, mut l: Fade);
    fn add_generic_action(&mut self, a: String, p: String);

    fn get_next_global_id(self) -> u32;
}

impl Actions for ActionGroups {
    fn add_delta_action(&mut self, mut m: Motion )
    {
        // Set the ID for the move being added to the set
        m.id = self.delta.len() as u32 + 1;

        self.delta.push( DeltaAction {
            id: self.global_id,
            action: String::from("queue_movement"),
            payload: m,
        } );

        self.global_id = self.global_id + 1;
    }

    fn add_light_action(&mut self, mut l: Fade)
    {
        l.id = self.light.len() as u32 + 1;

        self.light.push(
          LightAction {
              id: self.global_id,
              action: "queue_light".to_string(),
              payload: l,
              comment: "".to_string()
          }
        );

        self.global_id = self.global_id + 1;
    }

    fn add_generic_action(&mut self, a: String, p: String)
    {
        self.run.push(
            GenericAction {
                id: self.global_id,
                action: a,
                payload: p,
                comment: "".to_string(),
                wait_for: 0,
            }
        );

        self.global_id = self.global_id + 1;
    }

    fn get_next_global_id(self) -> u32 {
        return self.global_id
    }
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
