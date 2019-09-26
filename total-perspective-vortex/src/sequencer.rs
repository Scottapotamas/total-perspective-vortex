use crate::blender_ingest::*;
use crate::zaphod_export::*;

// Generate a move between A and B
fn move_between(a: BlenderPoint, b: BlenderPoint, speed: f32) -> Option<DeltaAction> {
    if a != b {
        let transit_points: Vec<(f32, f32, f32)> = vec![a, b]
            .iter()
            .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
            .collect();
        let transit_duration = calculate_duration(&[a, b], speed);

        return Some(DeltaAction {
            id: 0,
            action: String::from("transit"),
            payload: Motion {
                id: 0,
                reference: 0,
                motion_type: 1,
                duration: (transit_duration * 1000.0) as u32,
                points: transit_points,
            },
        });
    } else {
        return None;
    }
}

pub fn sequence_events(input: Vec<IlluminatedSpline>) -> ActionGroups {
    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut movement_events: Vec<DeltaAction> = Vec::new();
    let mut lighting_events: Vec<LightAction> = Vec::new();
    let mut additional_steps: Vec<GenericAction> = Vec::new();

    let mut last_point: BlenderPoint = BlenderPoint {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    // Apply transformations to the parsed data
    for spline_to_process in &input {
        let input_spline = &spline_to_process.spline;
        let input_colors = &spline_to_process.illumination;

        let mut spline_type = 0;
        let mut window_size = 0;

        match input_spline.spline_type.as_str() {
            "poly" => {
                spline_type = 1;
                window_size = 2;
            }
            "nurbs" => {
                spline_type = 2;
                window_size = 4;
            }
            _ => {
                println!(
                    "Unsupported blender data type: {}",
                    input_spline.spline_type.as_str()
                );
            }
        }

        // Generate a move from the end of the last spline to the start of the next spline
        let next_point = input_spline.points[0].clone();

        match move_between(last_point, next_point, 300.0) {
            Some(mut transit) => {
                transit.payload.id = movement_events.len() as u32;
                movement_events.push(transit);
            }
            _ => println!("No transit required"),
        }

        // Calculate movements to follow the line/spline
        for geometry in input_spline.points.windows(window_size) {
            // Calculate the duration of this move, and accumulate it for the whole spline
            let move_time = calculate_duration(geometry, 300.0);

            // Calculate the start and end times of the move in the set,
            let timestamp_begin = input_spline.target_duration;
            let timestamp_end = input_spline.target_duration + move_time;

            // resolve mutability/responsibility conflict?
            //            input_spline.target_duration += move_time;

            last_point = geometry[1];

            // Grab the xyz co-ords (discard blender's w term)
            let points_list: Vec<(f32, f32, f32)> = geometry
                .iter()
                .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
                .collect();

            movement_events.push(DeltaAction {
                id: 0,
                action: String::from("queue_movement"),
                payload: Motion {
                    id: movement_events.len() as u32,
                    reference: 0,
                    motion_type: spline_type,
                    duration: (move_time * 1000.0) as u32,
                    points: points_list,
                },
            });

            // Provide lighting matching this movement
            println!("Lighting to process for this spline");
        }
    // Assign all the moves, leds, and extra actions, and apply unique global ID's to all of them

    let mut event_identifier = 0;

    for movement in &mut movement_events {
        movement.id = event_identifier;
        event_identifier += 1;
    }

    for illumination in &mut lighting_events {
        illumination.id = event_identifier;
        event_identifier += 1;
    }

    for generic in &mut additional_steps {
        generic.id = event_identifier;
        event_identifier += 1;
    }

    // Prepare the data for export
    let event_set = ActionGroups {
        delta: movement_events,
        light: lighting_events,
        run: additional_steps,
    };

    return event_set;
}