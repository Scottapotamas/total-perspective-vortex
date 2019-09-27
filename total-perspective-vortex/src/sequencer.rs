use crate::blender_ingest::*;
use crate::zaphod_export::*;
use colorsys::Hsl;

const MOVEMENT_SPEED: f32 = 200.0;
const CLUSTER_THRESHOLD: f64 = 300.0;

// Generate a move between A and B
fn move_between(a: BlenderPoint, b: BlenderPoint, speed: f32) -> Option<DeltaAction> {
    if a != b {
        let transit_points: Vec<(f32, f32, f32)> = vec![a, b]
            .iter()
            .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
            .collect();
        let transit_duration = calculate_duration(&[a, b], speed).unwrap();

        return Some(DeltaAction {
            id: 0,
            action: String::from("queue_movement"),
            payload: Motion {
                id: 0,
                reference: 0,
                motion_type: 1,
                duration: transit_duration as u32,
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

        match move_between(last_point, next_point, MOVEMENT_SPEED) {
            Some(mut transit) => {
                transit.payload.id = movement_events.len() as u32;
                movement_events.push(transit);
            }
            _ => (),
        }

        let mut spline_time = 0.0;

        // Calculate movements to follow the line/spline
        for geometry in input_spline.points.windows(window_size) {
            // Calculate the duration of this move, and accumulate it for the whole spline
            let move_time = calculate_duration(geometry, MOVEMENT_SPEED).unwrap();
            spline_time = spline_time + move_time;

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
                    duration: move_time as u32,
                    points: points_list,
                },
            });
        }

        // Generate lighting events matching the UV for this movement
        let lighting_steps = input_colors.len();
        let step_duration = spline_time / lighting_steps as f32;

        // Keep track of the colour at the start of a given cluster
        let mut start_colour: (usize, &Hsl) = (0, &input_colors[0]);

        // Run through the gradient and generate planner fades between visually distinct colours
        // this effectively 'de-dupes' the command set for gentle gradients
        for (i, next_colour) in input_colors.iter().enumerate() {
            // Check if our tracked colour and this point are sufficiently visually different
            if distance_hsl(start_colour.1, next_colour).abs() > CLUSTER_THRESHOLD {
                // Calculate the duration of the interval between selected points
                let step_difference = i - start_colour.0;
                let fade_duration = step_difference as f32 * step_duration;

                // Grab and format [0,1] the colours into the delta-compatible tuple
                let cluster_start = (
                    start_colour.1.get_hue() as f32 / 360.0,
                    start_colour.1.get_saturation() as f32 / 100.0,
                    start_colour.1.get_lightness() as f32 / 100.0,
                );

                let cluster_end = (
                    next_colour.get_hue() as f32 / 360.0,
                    next_colour.get_saturation() as f32 / 100.0,
                    next_colour.get_lightness() as f32 / 100.0,
                );

                // Add the event to the lighting events pool
                let fade = LightAnimation {
                    animation_type: 1,
                    id: 2,
                    duration: fade_duration as u32,
                    points: vec![cluster_start, cluster_end],
                };

                lighting_events.push(LightAction {
                    id: 0,
                    action: "queue_light".to_string(),
                    payload: fade,
                    comment: "".to_string(),
                });

                // Set the 'end' of the fade to be the start of the next comparison
                start_colour.0 = i;
                start_colour.1 = next_colour;
            }
            // else
            // skip the colour because it's too similar to the tracked 'start' point
        }
    }

    // Assign all the moves, lights, extra actions a unique global ID, as json doesn't guarantee order
    let mut event_identifier = 0;

    movement_events.iter_mut().for_each(|movement| {
        movement.id = event_identifier;
        event_identifier += 1;
    });

    lighting_events.iter_mut().for_each(|illumination| {
        illumination.id = event_identifier;
        event_identifier += 1;
    });

    additional_steps.iter_mut().for_each(|generic| {
        generic.id = event_identifier;
        event_identifier += 1;
    });

    // Prepare the data for export
    let event_set = ActionGroups {
        delta: movement_events,
        light: lighting_events,
        run: additional_steps,
    };

    return event_set;
}