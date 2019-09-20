use std::path::Path;

extern crate walkdir;
use walkdir::{DirEntry, WalkDir};

pub mod blender_ingest;
use blender_ingest::*;

pub mod zaphod_export;
use zaphod_export::*;

// Checks that a DirEntry isn't hidden, a __MACOSX folder, or a file
fn is_frame_folder(entry: &DirEntry) -> bool {
    if entry.file_type().is_dir() {
        entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with(".") && !s.starts_with("__"))
            .unwrap_or(false)
    } else {
        return false;
    }
}

fn main() {
    println!("Welcome to the Total Perspective Vortex!");

    // Walk the folder structure looking for frame folders, then process them
    WalkDir::new("./multi")
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_frame_folder(e))
        .filter_map(|v| v.ok())
        .for_each(|x| process_frame_folder(&x));
}

fn is_json_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".json") && !s.ends_with("blahOut.json"))
        .unwrap_or(false)
}

// From a valid frame folder, find files, process, and generate the toolpath
fn process_frame_folder(entry: &DirEntry) {
    println!("\nProcessing Frame {:?}", entry.file_name());

    let mut parsed_splines = vec![];

    // Parse the json files
    WalkDir::new(entry.path())
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| is_json_file(e))
        .filter_map(|v| v.ok())
        .for_each(|x| parsed_splines.push(load_blender_data(&x.path())));

    // Prepare data for output
    //todo apply scaling/transforms to points here
    //todo apply illumination offsets here
    let output_data = sequence_events(parsed_splines);

    // Put the output JSON in the folder alongside the input files
    let export_json_name = Path::new("blahOut.json");
    let destination_path = entry.path().join(&export_json_name);

    // Write to the JSON file in format suitable for zaphod-bot
    save_toolpath(destination_path.as_path(), output_data);
}

fn sequence_events(input: Vec<IlluminatedSpline>) -> DeltaEvents {
    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut movement_events: Vec<DeltaAction> = Vec::new();
    let mut lighting_events: Vec<LightAction> = Vec::new();
    let mut additional_steps: Vec<GenericAction> = Vec::new();

    // Event Identifiers need to be unique and sequential across _any_ event in the file, regardless of device
    let mut event_identifier = 1;

    // Goes into the movement as the ID. Should be unique and increasing (doesn't require constant inc rate).
    let mut delta_move_identifier = 2;

    // Light events are tied against movement ID's, if many light animations are needed per movement,
    // then use the same ID
    let mut light_identifier = 2;

    // Apply transformations to the parsed data
    for ill_spline in &input {
        let input_spline = &ill_spline.spline;
        let input_colors = &ill_spline.illumination;

        // Calculate movement speed, durations etc
        let num_move_points = input_spline.points.len();
        let num_light_points = input_colors.len();
        let curve_length = input_spline.curve_length * 100.0; // in centimeters
        let effector_speed = 30.0 / 100.0; //centimeters/second
        let total_duration = curve_length / effector_speed;

        let move_segment_duration = total_duration / num_move_points as f32;
        let light_segment_duration =
            move_segment_duration / (num_light_points as f32 / num_move_points as f32);

        // Create a preparation move to the first point in the blender spline
        movement_events.push(generate_preparation_movement(transform_points(
            input_spline.points[1].0,
            input_spline.points[1].1,
            input_spline.points[1].2,
        )));

        event_identifier = event_identifier + 1;

        let mut running_length = 0;

        // Slice 4 points from the list (one catmull spline segment) and generate a valid toolpath event
        for catmull_segment in input_spline.points.windows(4) {
            // Grab the xyz co-ords (discard blender's w term), scale and offset as required
            let mut point_list: Vec<(f32, f32, f32)> = Vec::new();
            for position in catmull_segment {
                point_list.push(transform_points(position.0, position.1, position.2));
            }

            movement_events.push(DeltaAction {
                id: event_identifier,
                action: String::from("queue_movement"),
                payload: Motion {
                    id: delta_move_identifier,
                    reference: 0,
                    motion_type: 2,
                    duration: move_segment_duration as u32,
                    points: point_list,
                },
            });

            running_length = running_length + move_segment_duration as u32;
            delta_move_identifier = delta_move_identifier + 1;
            event_identifier = event_identifier + 1;
        }

        //        println!("The total move duration is {}", running_length);

        let mut multi_light_counter = 0;
        let mut running_length_light = 0;

        // Segment the input LED sequence into animations which have a total duration matching the curve
        // Each 'animation' is a linear fade between two colours, so a 2-element window slides across the input
        // Because there are more LED events than moves, ensure that the light_identifier doesn't exceed the movement ID for a given point in time
        for light_pair in input_colors.windows(2) {
            let mut gradient: Vec<(f32, f32, f32)> = Vec::new();

            for light in light_pair {
                // delta expects these values as [0-1] floats...
                gradient.push((
                    (light.get_hue() / 360.0) as f32,
                    (light.get_saturation() / 100.0) as f32,
                    (light.get_lightness() / 100.0) as f32,
                ));
            }

            lighting_events.push(LightAction {
                id: event_identifier,
                action: String::from("queue_light"),
                payload: LightAnimation {
                    animation_type: 1,
                    id: 2 as u32, //light_identifier,
                    duration: light_segment_duration as u32,
                    points: gradient,
                },
                comment: String::from(""),
            });

            //        multi_light_counter = multi_light_counter + 1;
            //
            //        if multi_light_counter > (num_light_points / num_move_points) {
            //            light_identifier = light_identifier + 1;
            //            multi_light_counter = 0;
            //        }
            running_length_light = running_length_light + light_segment_duration as u32;
            event_identifier = event_identifier + 1;
        }
        //        println!("The total light duration is {}", running_length_light);
    }

    let bootstrap_sync = GenericAction {
        id: event_identifier,
        action: String::from("sync"),
        payload: String::from("1"),
        comment: String::from("triggering move"),
        wait_for: (event_identifier - 1),
    };
    additional_steps.push(bootstrap_sync);
    event_identifier = event_identifier + 1;

    let camera_trigger = GenericAction {
        id: event_identifier,
        action: String::from("capture"),
        payload: String::from("{ \"filePath\": \"./vortex/{{time}}.jpg\" }"),
        comment: String::from("capturing action"),
        wait_for: (event_identifier - 1),
    };
    additional_steps.push(camera_trigger);
    event_identifier = event_identifier + 1;

    let event_set: DeltaEvents = DeltaEvents {
        metadata: generate_header(String::from("VortexFile")),
        actions: vec![ActionGroups {
            delta: movement_events,
            light: lighting_events,
            run: additional_steps,
        }],
    };

    return event_set;
}
