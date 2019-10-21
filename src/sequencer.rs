use crate::color_utils::*;
use crate::delta_utils::*;

use crate::export_types::*;
use crate::import_types::*;

use colorsys::Hsl;

const MOVEMENT_SPEED: f32 = 200.0;
const CLUSTER_THRESHOLD: f64 = 300.0;

// Returns
//      motion type (used in the delta as the pathing engine selector)
//      window size to scan (how many points are needed for a given minimal event),
//      coordinate index which is the 'first' point of the spline
//      coordinate index which is the 'last' point of the spline
fn spline_type_selector(spline_type: &str) -> Option<(u32, usize, usize, usize)> {
    match spline_type {
        "poly" => {
            return Some((1, 2, 0, 1));
        }
        "nurbs" => {
            return Some((2, 4, 1, 2));
        }
        _ => {
            println!("Unsupported blender data type: {}", spline_type);
            return None;
        }
    }
}

// Generate a move between A and B
fn move_between(a: BlenderPoint4, b: BlenderPoint4, speed: f32) -> Option<DeltaAction> {
    if a != b {
        // Generate transit move instead of requiring a start from home
        if a.x == 0.0 && a.y == 0.0 && a.z == 0.0 {
            return Some(DeltaAction {
                id: 0,
                action: String::from("queue_movement"),
                payload: Motion {
                    id: 0,
                    reference: 0,
                    motion_type: 0,
                    duration: 500,
                    points: vec![(b.x, b.y, b.z)],
                },
            });
        }

        let transit_points: Vec<(f32, f32, f32)> = vec![a, b]
            .iter()
            .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
            .collect();
        let transit_duration = calculate_duration(&[a, b], speed).unwrap() as u32;

        return Some(DeltaAction {
            id: 0,
            action: String::from("queue_movement"),
            payload: Motion {
                id: 0,
                reference: 0,
                motion_type: 1,
                duration: transit_duration,
                points: transit_points,
            },
        });
    } else {
        return None;
    }
}

pub fn generate_delta_toolpath(input: &Vec<BlenderData>) -> ActionGroups {
    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut movement_events: Vec<DeltaAction> = Vec::new();
    let mut lighting_events: Vec<LightAction> = Vec::new();
    let mut additional_steps: Vec<GenericAction> = Vec::new();

    let mut last_point: BlenderPoint4 = BlenderPoint4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    // Apply transformations to the parsed data
    for spline_to_process in input {
        let input_spline = &spline_to_process.spline;
        let input_colors = &spline_to_process.illumination;

        let (spline_type, window_size, spline_start_index, spline_finish_index) =
            spline_type_selector(input_spline.spline_type.as_str()).unwrap();

        // Generate a move from the end of the last spline to the start of the next spline
        let next_point = input_spline.points[spline_start_index].clone();

        match move_between(last_point, next_point, MOVEMENT_SPEED) {
            Some(mut transit) => {
                // Add the event to the lighting events pool
                let fade = LightAnimation {
                    animation_type: 1,
                    id: 1,
                    duration: transit.payload.duration as f32,
                    points: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.0)],
                };

                lighting_events.push(LightAction {
                    id: 0,
                    action: "queue_light".to_string(),
                    payload: fade,
                    comment: "".to_string(),
                });

                transit.payload.id = movement_events.len() as u32 + 1;
                movement_events.push(transit);
            }
            _ => (),
        }

        let mut spline_time = 0;

        // Calculate movements to follow the line/spline
        for geometry in input_spline.points.windows(window_size) {
            // Calculate the duration of this move, and accumulate it for the whole spline
            let move_time = calculate_duration(geometry, MOVEMENT_SPEED).unwrap() as u32;
            spline_time = spline_time + move_time;

            last_point = geometry[spline_finish_index];

            // Grab the xyz co-ords (discard blender's w term)
            let points_list: Vec<(f32, f32, f32)> = geometry
                .iter()
                .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
                .collect();

            movement_events.push(DeltaAction {
                id: 0,
                action: String::from("queue_movement"),
                payload: Motion {
                    id: movement_events.len() as u32 + 1,
                    reference: 0,
                    motion_type: spline_type,
                    duration: move_time,
                    points: points_list,
                },
            });
        }

        // Generate lighting events matching the UV for this movement
        let lighting_steps = input_colors.len() - 1;
        let step_duration = spline_time as f32 / lighting_steps as f32;

        // Keep track of the colour at the start of a given cluster
        let mut start_colour: (usize, &Hsl) = (0, &input_colors[0]);

        let mut sum_lighting_time = 0.0;

        // Run through the gradient and generate planner fades between visually distinct colours
        // this effectively 'de-dupes' the command set for gentle gradients
        for (i, next_colour) in input_colors.iter().enumerate() {
            // Check if our tracked colour and this point are sufficiently visually different
            if distance_hsl(start_colour.1, next_colour).abs() > CLUSTER_THRESHOLD
                || lighting_steps < 3 && i != 0
            {
                // Calculate the duration of the interval between selected points
                let step_difference = i - start_colour.0;
                let fade_duration = step_difference as f32 * step_duration;

                // Grab and format [0,1] the colours into the delta-compatible tuple
                let cluster_start = delta_led_from_hsl(start_colour.1);
                let cluster_end = delta_led_from_hsl(next_colour);

                // Add the event to the lighting events pool
                let fade = LightAnimation {
                    animation_type: 1,
                    id: 1,
                    duration: fade_duration,
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

                sum_lighting_time = sum_lighting_time + fade_duration;
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

// The viewer preview data consists of line segments and a UV map
pub fn generate_viewer_data(input: &Vec<IlluminatedSpline>) -> (Vec<(f32, f32, f32)>, Vec<Hsl>) {
    let mut poly_points = vec![];
    let mut uv_colors = vec![];

    // Apply transformations to the parsed data
    for spline_to_process in input {
        let input_spline = &spline_to_process.spline;
        let input_colors = &spline_to_process.illumination;

        let (spline_type, window_size, spline_start_index, spline_finish_index) =
            spline_type_selector(input_spline.spline_type.as_str()).unwrap();

        // Generate a move from the end of the last spline to the start of the next spline
        let next_point = input_spline.points[0].clone();

        // create transition movement
        //        poly_points.push(blah);

        let mut spline_time = 0.0;

        // Calculate movements to follow the line/spline
        for geometry in input_spline.points.windows(window_size) {
            // Calculate the duration of this move, and accumulate it for the whole spline
            let move_time = calculate_duration(geometry, MOVEMENT_SPEED).unwrap();
            spline_time = spline_time + move_time;

            poly_points.extend(vertex_from_spline(spline_type, geometry));
        }

        uv_colors.extend(input_colors.clone());
    }

    return (poly_points, uv_colors);
}
