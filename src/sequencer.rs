use crate::color_utils::*;
use crate::delta_utils::*;

use crate::export_types::*;
use crate::import_types::*;

use colorsys::Hsl;

const MOVEMENT_SPEED: f32 = 200.0;
const CLUSTER_THRESHOLD: f64 = 300.0;


// Generate a move between A and B
fn move_between(a: BlenderPoint4, b: BlenderPoint4, speed: f32) -> Option<Motion> {
    if a != b {
        // Generate transit move instead of requiring a start from home
        if a.x == 0.0 && a.y == 0.0 && a.z == 0.0 {
            return Some(Motion {
                    id: 0,
                    reference: 0,
                    motion_type: 0,
                    duration: 500,
                    points: vec![(b.x, b.y, b.z)],
            });
        }

        let transit_points: Vec<(f32, f32, f32)> = vec![a, b]
            .iter()
            .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
            .collect();

        let transit_duration = calculate_duration(&[a, b], speed).unwrap() as u32;

        return Some( Motion {
                id: 0,
                reference: 0,
                motion_type: 1,
                duration: transit_duration,
                points: transit_points,
        });
    } else {
        return None;
    }
}

fn add_starting_move( events: &mut ActionGroups , a: BlenderPoint4, b: BlenderPoint4,)
{
    match move_between(a, b, MOVEMENT_SPEED) {
        Some( transit) => {
            // Also add an equal duration lighting event so we have a unlit transit
            events.add_light_action( Fade {
                animation_type: 1,
                id: 0,
                duration: transit.duration as f32,
                points: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.0)],
            } );

            events.add_delta_action( transit );
        }
        _ => (),
    }
}

pub fn generate_delta_toolpath(input: &Vec<BlenderData>) -> ActionGroups {
    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut event_set = ActionGroups::new();
    
    let mut last_point: BlenderPoint4 = BlenderPoint4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    // Apply transformations to the parsed data
    for spline_to_process in input {


        match &spline_to_process {
            BlenderData::PolySpline(spline) => {

                // Generate a move from the end of the last spline to the start of the next spline
                let next_point = spline.points[0].clone();

                add_starting_move( &mut event_set, last_point, next_point );

                // Polysplines are a chain of lines, a line consists of a pair of BlenderPoint4 co-ordinates
                for geometry in spline.points.windows(BlenderPoly::get_recommended_window_size()) {
                    // Calculate the duration of this move, and accumulate it for the whole spline
                    let move_time = calculate_duration(geometry, MOVEMENT_SPEED).unwrap() as u32;

                    last_point = BlenderPoly::get_end_point(geometry);

                    // Grab the xyz co-ords (discard blender's w term)
                    let points_list: Vec<(f32, f32, f32)> = geometry
                        .iter()
                        .map(|bpoint| return (bpoint.x, bpoint.y, bpoint.z))
                        .collect();

                    event_set.add_delta_action(Motion {
                        id: 0,
                        reference: 0,
                        motion_type: 1, // polysplines are linear moves
                        duration: move_time,
                        points: points_list,
                    });
                }

                // Generate lighting events matching the UV for this movement
                let lighting_steps = spline.color.len() - 1;
                let step_duration = event_set.get_movement_duration() as f32 / lighting_steps as f32;

                // Keep track of the colour at the start of a given cluster
                let mut start_colour: (usize, &Hsl) = (0, &spline.color[0]);

                let mut sum_lighting_time = 0.0;

                // Run through the gradient and generate planner fades between visually distinct colours
                // this effectively 'de-dupes' the command set for gentle gradients
                for (i, next_colour) in spline.color.iter().enumerate() {
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
                        event_set.add_light_action(Fade {
                            animation_type: 1,
                            id: 1,
                            duration: fade_duration,
                            points: vec![cluster_start, cluster_end],
                        });

                        // Set the 'end' of the fade to be the start of the next comparison
                        start_colour.0 = i;
                        start_colour.1 = next_colour;

                        sum_lighting_time = sum_lighting_time + fade_duration;
                    }
                    // else
                    // skip the colour because it's too similar to the tracked 'start' point
                }


            },
            BlenderData::NURBSSpline(spline) => {
                println!("Wouldn't it be great if NURBS were supported...");
                // TODO Handle nurbs data

            },
            BlenderData::Particles( p) => {

                for particle in &p.particles {

                    // Create a movement from last to current with the specified colour


                }

            }
            _ => {
                // Unknown type. Do nothing
            }
        }


// The viewer preview data consists of line segments and a UV map
pub fn generate_viewer_data(input: &Vec<BlenderData>) -> (Vec<(f32, f32, f32)>, Vec<Hsl>) {
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



    }

    return event_set;
}

    return (poly_points, uv_colors);
}
