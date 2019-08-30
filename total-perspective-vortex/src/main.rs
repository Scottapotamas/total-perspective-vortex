use std::path::Path;

pub mod blender_ingest;
use blender_ingest::*;

pub mod zaphod_export;
use zaphod_export::*;

fn main() {
    println!("Welcome to the Total Perspective Vortex!");

    let destination_path = Path::new("./toolpath.json");
    let path_uv = Path::new("./spiral_sphere/2/spiral_sphere_uv.png");
    let path_json = Path::new("./spiral_sphere/2/spiral_sphere.json");
    //    let path_uv = Path::new("./simple_circle/simple_circle_uv.png");
    //    let path_json = Path::new("./simple_circle/simple_circle.json");

    let input_colors = load_uv(path_uv);

    let input_spline = load_json(path_json);

    // A delta-ready toolpath file has sets of events grouped by device (delta, led light, cameras etc).
    let mut movement_events: Vec<DeltaAction> = Vec::new();
    let mut lighting_events: Vec<LightAction> = Vec::new();
    let mut additional_steps: Vec<GenericAction> = Vec::new();

    // Calculate movement speed, durations etc
    let num_move_points = input_spline.points.len();
    let num_light_points = input_colors.len();
    let curve_length = input_spline.curve_length * 100.0; // in centimeters
    let effector_speed = 20.0 / 100.0; //centimeters/second
    let total_duration = curve_length / effector_speed;

    let move_segment_duration = total_duration / num_move_points as f32;
    let light_segment_duration =
        move_segment_duration / (num_light_points as f32 / num_move_points as f32);

    // Event Identifiers need to be unique and sequential across _any_ event in the file, regardless of device
    let mut event_identifier = 1;

    // Goes into the movement as the ID. Should be unique and increasing (doesn't require constant inc rate).
    let mut delta_move_identifier = 2;

    // Light events are tied against movement ID's, if many light animations are needed per movement,
    // then use the same ID
    let mut light_identifier = 2;

    // Create a preparation move to the first point in the blender spline
    movement_events.push(generate_preparation_movement(transform_points(
        &input_spline.points[1].point[0..3],
    )));

    event_identifier = event_identifier + 1;

    // Slice 4 points from the list (one catmull spline segment) and generate a valid toolpath event
    for catmull_segment in input_spline.points.windows(4) {
        // Grab the xyz co-ords (discard blender's w term), scale and offset as required
        let mut point_list: Vec<(f32, f32, f32)> = Vec::new();
        for position in catmull_segment {
            point_list.push(transform_points(&position.point[0..3]));
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
        delta_move_identifier = delta_move_identifier + 1;
        event_identifier = event_identifier + 1;
    }

    let mut multi_light_counter = 0;

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

        event_identifier = event_identifier + 1;
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

    let output_data: DeltaEvents = DeltaEvents {
        metadata: generate_header(String::from("VortexFile")),
        actions: vec![ActionGroups {
            delta: movement_events,
            light: lighting_events,
            run: additional_steps,
        }],
    };

    // Pack the data structures into json for the delta, save to disk
    save_toolpath(destination_path, output_data);
}

/*
fn main() -> std::io::Result<()> {

    let test_directory = "./spiral_sphere";

    // Walk the folders, each should be a 'frame' of the animation
    for frame_folder in fs::read_dir(test_directory).expect("Unable to list folders") {
        // open the files for the current frame
        for frame_file in
            fs::read_dir(frame_folder.unwrap().path()).expect("Unable to files in folder")
        {
            println!("Directory has extension {:?}:", frame_file?.path());

            // Load a file
            //            let file = File::open("../../foo.txt")?;

            //            println!("File path {:?}", frame_file?.file_type());
            //            let json_file_path = Path::new(frame_file);

            //    assert_eq!(contents, "Hello, world!");
            //            println!("File contents are {}", contents);
        }

        // Folder contains a given 'frame', json and uv map
    }
    Ok(())
}
*/
