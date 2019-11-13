use crate::import_types::*;
use std::f32::MAX;

// Find a point partially between two points
pub fn interpolate_line_point(
    a: &BlenderPoint3,
    b: &BlenderPoint3,
    weight: f32,
) -> Result<BlenderPoint3, String> {
    if weight <= 0.0 || weight >= 1.0 {
        // Weights should be between 0.0-1.0 representing the percentage point to interpolate
        return Err("Can't interpolate point from line with input weight".to_string());
    }

    Ok(BlenderPoint3 {
        x: a.x + ((b.x - a.x) * weight),
        y: a.y + ((b.y - a.y) * weight),
        z: a.z + ((b.z - a.z) * weight),
    })
}

// Calculate the 2D distance in mm between two points
fn distance_2d(a: &BlenderPoint2, b: &BlenderPoint2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let distance = ((dx * dx) + (dy * dy)).sqrt();

    distance.abs()
}

// Calculate the 3D distance in mm between two points
fn distance_3d(a: &BlenderPoint3, b: &BlenderPoint3) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    let distance = ((dx * dx) + (dy * dy) + (dz * dz)).sqrt();

    distance.abs()
}

pub fn interpolate_catmull_point(
    p: &[BlenderPoint3],
    weight: f32,
) -> Result<BlenderPoint3, String> {
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

    Ok(BlenderPoint3 {
        x: out_x,
        y: out_y,
        z: out_z,
    })
}

// Estimate the 3D length of a catmull-rom spline by sampling repeatedly
fn distance_catmull(control_points: &[BlenderPoint3]) -> Result<f32, String> {
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

pub fn calculate_duration(points: &[BlenderPoint3], speed: f32) -> Result<f32, String> {
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

    let mut duration = (distance / speed) * 1000.0; // in milliseconds

    if duration < 10.0 {
        duration = 10.0;
    }

    Ok(duration)
}

pub fn vertex_from_spline(spline_type: u32, geometry: &[BlenderPoint3]) -> Vec<(f32, f32, f32)> {
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

    points_list
}

fn is_point_in_circle(point: &BlenderPoint2, circle_center: &BlenderPoint2, radius: f32) -> bool {
    distance_2d(point, circle_center) <= radius
}

pub fn is_point_legal(point: &BlenderPoint3) -> bool {
    let cylinder_offset = BlenderPoint2 { x: 0.0, y: 0.0 };
    let cylinder_ends: (f32, f32) = (0.0, 200.0);
    let cylinder_radius: f32 = 200.0;

    is_point_in_circle(&point.into_bp2_xy(), &cylinder_offset, cylinder_radius)
        && point.z > cylinder_ends.0
        && point.z < cylinder_ends.1
}

// Sort the particles into a chain of next-nearest distances to reduce the traversal distance for particle systems
// Very naiive approach - TODO solve travelling salesman problem!
pub fn sort_particles(particles: &mut Vec<BlenderParticle>) -> Vec<BlenderParticle> {
    let mut sorted_particles = vec![];
    sorted_particles.reserve(particles.len());

    // Randomly pick a starting point, Non-deterministic pathing through the particle cloud is desirable
    // as it should 'fuzz' any artifacts influenced by transit moves.
    let random_start = (rand::random::<f32>() * particles.len() as f32).floor() as usize;
    sorted_particles.push(particles.remove(random_start));

    while !particles.is_empty() {
        let mut closest_dist = MAX;
        let mut closest_index = None;
        let check_p = sorted_particles.last().unwrap(); // search from the most recent sorted point

        for (i, search_p) in particles.iter().enumerate() {
            let dist = distance_3d(&check_p.location, &search_p.location);

            if dist < closest_dist {
                closest_dist = dist;
                closest_index = Some(i);
            }
        }

        // Take the closest point from this search pass, and move it into the sorted vector
        if let Some(sorted_point_index) = closest_index {
            sorted_particles.push(particles.remove(sorted_point_index));
        }
    }

    sorted_particles
}
