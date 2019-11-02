use crate::color_utils::*;
use crate::delta_utils::*;

use crate::export_types::*;
use crate::import_types::*;

use colorsys::Hsl;

const MOVEMENT_SPEED: f32 = 200.0;
const CLUSTER_THRESHOLD: f64 = 300.0;


// Generate a move between A and B
fn move_between(a: BlenderPoint3, b: BlenderPoint3, speed: f32) -> Option<Motion> {
    if a != b {
        // Generate transit move instead of requiring a start from home
        if a.x == 0.0 && a.y == 0.0 && a.z == 0.0 {
            Some(Motion {
                    id: 0,
                    reference: 0,
                    motion_type: 0,
                    duration: 500,
                    points: vec![(b.x, b.y, b.z)],
            });
        }

        let transit_points: Vec<(f32, f32, f32)> = vec![a, b]
            .iter()
            .map(|bpoint| (bpoint.x, bpoint.y, bpoint.z))
            .collect();

        let transit_duration = calculate_duration(&[a, b], speed).unwrap() as u32;

        Some( Motion {
                id: 0,
                reference: 0,
                motion_type: 1,
                duration: transit_duration,
                points: transit_points,
        })
    } else {
        None
    }
}

fn add_starting_move( events: &mut ActionGroups , a: BlenderPoint3, b: BlenderPoint3,)
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

fn add_delay( events: &mut ActionGroups, time: u32)
{
    // Abuse the relative movement to move 'zero distance' over time
    events.add_delta_action( Motion {
        motion_type: 0,
        reference: 1,
        id: 0,
        duration: time,
        points: vec![(0.0, 0.0, 0.0)]
    } );

    // Also add an equal duration lighting event
    events.add_light_action( Fade {
        animation_type: 1,
        id: 0,
        duration: time as f32,
        points: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.0)],
    } );

}

// Generates lighting 'fade' events between the (expanding until visually different) edges of the colour vector slice
fn generate_visually_distinct_fade<'a>( events: &mut ActionGroups, i: usize, steps: usize, duration:f32, start_colour: (usize, &'a Hsl), next_colour: (usize, &'a Hsl),  ) -> (usize, &'a Hsl)
{
    if distance_hsl(start_colour.1, next_colour.1).abs() > CLUSTER_THRESHOLD || steps < 3 || i == steps && i != 0
    {
        // Calculate the duration of the interval between selected points
        let step_difference = i - start_colour.0;
        let fade_duration = step_difference as f32 * duration;

        // Grab and format [0,1] the colours into the delta-compatible tuple
        let cluster_start = delta_led_from_hsl(start_colour.1);
        let cluster_end = delta_led_from_hsl(next_colour.1);

        // Add the event to the lighting events pool
        events.add_light_action(Fade {
            animation_type: 1,
            id: 1,
            duration: fade_duration,
            points: vec![cluster_start, cluster_end],
        });

        // Set the 'end' of the fade to be the start of the next comparison
        return next_colour;
    }

    return start_colour;
}

pub fn generate_delta_toolpath(input: &[BlenderData]) -> ActionGroups {
    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut event_set = ActionGroups::new();
    
    let mut last_point: BlenderPoint3 = BlenderPoint3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    // Apply transformations to the parsed data
    for spline_to_process in input {


        match &spline_to_process {
            BlenderData::PolySpline(spline) => {

                // Generate a move from the end of the last spline to the start of the next spline
                add_starting_move( &mut event_set, last_point, spline.points[0].into_bp3() );

                // Polysplines are a chain of lines, a line consists of a pair of BlenderPoint co-ordinates
                for geometry in spline.points.windows(BlenderPoly::get_recommended_window_size()) {

                    let geom: [BlenderPoint3;2] = [geometry[0].into_bp3(), geometry[1].into_bp3() ];

                    let duration = calculate_duration(&geom, MOVEMENT_SPEED).unwrap() as u32;
                    last_point = BlenderPoly::get_end_point(geometry).into_bp3();
                    event_set.add_delta_action(Motion {
                        id: 0,
                        reference: 0,
                        motion_type: 1, // polysplines are linear moves
                        duration,
                        points: geom.iter().map(|bpoint| (bpoint.x, bpoint.y, bpoint.z)).collect(), // Grab a xyz co-ord tuple
                    });
                }

                // Generate lighting events matching the UV for this movement
                let lighting_steps = spline.color.len() - 1;
                let step_duration = event_set.get_movement_duration() as f32 / lighting_steps as f32;

                // Keep track of the colour at the start of a given cluster
                let mut start_colour: (usize, &Hsl) = (0, &spline.color[0]);

                // Run through the gradient and generate planner fades between visually distinct colours
                // this effectively 'de-dupes' the command set for gentle gradients
                for (i, next_colour) in spline.color.iter().enumerate() {
                    start_colour = generate_visually_distinct_fade( &mut event_set, i, lighting_steps, step_duration, start_colour, (i, next_colour) );
                }

            },
            BlenderData::NURBSSpline(spline) => {
                println!("Wouldn't it be great if NURBS were supported...");
                // TODO Handle nurbs data

            },
            BlenderData::Particles( p) => {

                // Create a movement for each particle between last and current locations with the specified 'global' colour
                for particle in &p.particles {

                    // Move to the particle's start point
                    add_starting_move( &mut event_set, last_point, particle.prev_location );

                    // We want to execute a line over the length of the particle's trail
                    let p_line = [particle.prev_location, particle.location];
                    let move_duration = calculate_duration(&p_line, MOVEMENT_SPEED).unwrap() as u32;

                    last_point = particle.location; //retain this for use in the next loop's transit start

                    add_delay(&mut event_set,50);

                    event_set.add_delta_action(Motion {
                        id: 0,
                        reference: 0,
                        motion_type: 1, // particle trails are linear moves
                        duration: move_duration,
                        points: p_line.iter().map(|p| (p.x, p.y, p.z)).collect(),
                    });

                    let p_color = p.color.iter().map(|c| delta_led_from_hsl(c)).collect();
                    event_set.add_light_action(Fade{
                        animation_type: 0,
                        id: 0,
                        duration: move_duration as f32,
                        points: p_color,
                    });
                }


                
            }
        }

    }

    event_set
}

// The viewer preview data consists of line segments and a UV map
pub fn generate_viewer_data(input: &[BlenderData]) -> (Vec<(f32, f32, f32)>, Vec<Hsl>) {
    let mut poly_points = vec![];
    let mut uv_colors = vec![];

    // Apply transformations to the parsed data
    for spline_to_process in input {
        match &spline_to_process {
            BlenderData::PolySpline(s) => {

                // Calculate movements to follow the line/spline
                for geometry in s.points.windows(BlenderPoly::get_recommended_window_size()) {
                    let geom: [BlenderPoint3;2] = [geometry[0].into_bp3(), geometry[1].into_bp3() ];

                    poly_points.extend(vertex_from_spline(1, &geom));
                }

                uv_colors.extend(s.color.clone());

            },
            BlenderData::NURBSSpline(s) => {

                println!("NURBS unavailable in preview...");
                poly_points.push((0.0,0.0,0.0));
                poly_points.push((0.0,0.0,0.0));
                uv_colors.push( Hsl::new(0.0, 0.0, 50.0, Option::from(1.0)) );
                uv_colors.push( Hsl::new(0.0, 0.0, 50.0, Option::from(1.0)) );

            },
            BlenderData::Particles(p) => {

                println!("Particles unavailable in preview...");
                poly_points.push((0.0,0.0,0.0));
                poly_points.push((0.0,0.0,0.0));

                uv_colors.push( Hsl::new(0.0, 0.0, 50.0, Option::from(1.0)) );
                uv_colors.push( Hsl::new(0.0, 0.0, 50.0, Option::from(1.0)) );

            },
        }

    }

    (poly_points, uv_colors)
}
