use crate::import_types::*;

// Calculate the 3D distance in mm between two points
fn distance_3d(a: &BlenderPoint4, b: &BlenderPoint4) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    let distance = ((dx * dx) + (dy * dy) + (dz * dz)).sqrt();

    return distance.abs();
}

pub fn interpolate_catmull_point(p: &[BlenderPoint4], weight: f32) -> Result<BlenderPoint4, String> {
    if weight <= 0.0 || weight >= 1.0 {
        // Weights should be between 0.0-1.0 representing the percentage point to interpolate
        return Err("Can't interpolate catmull with input weight".to_string());
    }

    let t = weight;
    let t2 = t * t;
    let t3 = t2 * t;

    /* Derivation from http://www.mvps.org/directx/articles/catmull/

                                [  0  2  0  0 ]   [ p0 ]
    q(t) = 0.5( t, t^2, t^3 ) * [ -1  0  1  0 ] * [ p1 ]
                                [  2 -5  4 -1 ]   [ p2 ]
                                [ -1  3 -3  1 ]   [ p3 ]
     */

    let out_x = 0.5
        * ((2.0 * p[1].x)
            + (-p[0].x + p[2].x) * t
            + (2.0 * p[0].x - 5.0 * p[1].x + 4.0 * p[2].x - p[3].x) * t2
            + (-p[0].x + 3.0 * p[1].x - 3.0 * p[2].x + p[3].x) * t3);

    let out_y = 0.5
        * ((2.0 * p[1].y)
            + (-p[0].y + p[2].y) * t
            + (2.0 * p[0].y - 5.0 * p[1].y + 4.0 * p[2].y - p[3].y) * t2
            + (-p[0].y + 3.0 * p[1].y - 3.0 * p[2].y + p[3].y) * t3);

    let out_z = 0.5
        * ((2.0 * p[1].z)
            + (-p[0].z + p[2].z) * t
            + (2.0 * p[0].z - 5.0 * p[1].z + 4.0 * p[2].z - p[3].z) * t2
            + (-p[0].z + 3.0 * p[1].z - 3.0 * p[2].z + p[3].z) * t3);

    Ok(BlenderPoint4 {
        x: out_x,
        y: out_y,
        z: out_z,
        w: 0.0,
    })
}

// Estimate the 3D length of a catmull-rom spline by sampling repeatedly
fn distance_catmull(control_points: &[BlenderPoint4]) -> Result<f32, String> {
    let samples: Vec<u32> = (1..99).collect();

    let length: f32 = samples
        .windows(2)
        .map(|p| {
            let a = interpolate_catmull_point(control_points, p[0] as f32 * 0.01).unwrap();
            let b = interpolate_catmull_point(control_points, p[1] as f32 * 0.01).unwrap();
            distance_3d(&a, &b)
        })
        .sum();

    Ok(length)
}

pub fn calculate_duration(points: &[BlenderPoint4], speed: f32) -> Result<f32, String> {
    let distance;

    match points.len() {
        1 => return Err("Duration for one point?".to_string()),
        2 => {
            distance = distance_3d(&points[0], &points[1]);
        }
        4 => {
            distance = distance_catmull(points)?;
        }
        _ => return Err("Can't calculate duration on this number of points".to_string()),
    }

    let duration = (distance / speed) * 1000.0; // in milliseconds

    Ok(duration)
}

pub fn vertex_from_spline(spline_type: u32, geometry: &[BlenderPoint4]) -> Vec<(f32, f32, f32)> {
    let mut points_list: Vec<(f32, f32, f32)> = vec![];

    // take the two points of the line, or sample points from the catmull chain
    match spline_type {
        1 => {
            // Grab the xyz co-ords (discard blender's w term)
            for point in geometry {
                points_list.push((point.x, point.y, point.z));
            }
        }
        2 => {
            let samples: Vec<u32> = (1..99).collect();

            for sample in samples {
                let point = interpolate_catmull_point(geometry, sample as f32 * 0.01).unwrap();
                points_list.push((point.x, point.y, point.z));
            }
        }
        _ => println!("Error generating preview vertices for unknown spline type"),
    }

    return points_list;
}
