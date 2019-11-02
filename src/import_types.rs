use colorsys::Hsl;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum BlenderData {
    #[serde(rename = "poly")]
    PolySpline(BlenderPoly),
    #[serde(rename = "nurbs")]
    NURBSSpline(BlenderNURBS),
    #[serde(rename = "particles")]
    Particles(BlenderParticles),
}

pub trait Spline {

    fn close_loop(&mut self) {
        // optional - particle systems don't have loops
    }

    fn scale_points(&mut self, factor: f32);
    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32);

    // The edges of the movement (not the first or last element, as those are often control points)
    fn get_start_point(slice: &[BlenderPoint4]) -> BlenderPoint4;
    fn get_end_point(slice: &[BlenderPoint4]) -> BlenderPoint4;

    // Size of the window to slide through the points
    fn get_recommended_window_size() -> usize;
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlenderPoly {
    pub curve_length: f32,
    pub points: Vec<BlenderPoint4>,
    #[serde(default)]
    pub cyclic: bool,
    #[serde(rename = "uv")]
    pub uv_path: String,

    #[serde(skip)]
    pub color: Vec<Hsl>,
}

impl Spline for BlenderPoly {

    fn close_loop(&mut self) {
        if self.cyclic {
            // Put the first point at the end of the set
            self.points.push( self.points.first().unwrap().clone() );
        }
    }

    fn scale_points(&mut self, factor: f32) {
        for p in &mut self.points {
            p.scale(factor)
        }

        self.curve_length *= factor;
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        for point in &mut self.points {
            point.offset(x_offset, y_offset, z_offset);
        }
    }

    fn get_start_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        slice[0]
    }

    fn get_end_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        slice[1]
    }

    fn get_recommended_window_size() -> usize {
        2
    }

}

#[derive(Deserialize, Debug, Clone)]
pub struct BlenderNURBS {
    pub curve_length: f32,
    pub points: Vec<BlenderPoint4>,
    #[serde(default)]
    pub cyclic: bool,
    #[serde(rename = "uv")]
    pub uv_path: String,

    #[serde(skip)]
    pub color: Vec<Hsl>,
}

impl Spline for BlenderNURBS {

    fn close_loop(&mut self) {
        if self.cyclic {
            // Put the first two points at the end of the set
            self.points.push(self.points[0]);
            self.points.push(self.points[1]);
            self.points.push(self.points[2]);

        }
    }

    fn scale_points(&mut self, factor: f32) {
        for p in &mut self.points {
            p.scale(factor)
        }

        self.curve_length *= factor;
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        for point in &mut self.points {
            point.offset(x_offset, y_offset, z_offset);
        }
    }

    fn get_start_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        slice[1]
    }

    fn get_end_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        slice[2]
    }

    fn get_recommended_window_size() -> usize {
        4
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlenderParticles {
    pub particles: Vec<BlenderParticle>,
    #[serde(rename = "color")]
    pub color_rgba: (f32,f32,f32,f32),

    #[serde(skip)]
    pub color: Vec<Hsl>,
}

impl Spline for BlenderParticles {

    fn close_loop(&mut self) {
        unimplemented!();
    }

    fn scale_points(&mut self, factor: f32) {
        for p in &mut self.particles {
            p.scale(factor)
        }
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32 ) {
        for particle in &mut self.particles {
            particle.offset(x_offset,y_offset,z_offset);
        }
    }

    fn get_start_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        unimplemented!()
    }

    fn get_end_point(slice: &[BlenderPoint4]) -> BlenderPoint4 {
        unimplemented!()
    }

    fn get_recommended_window_size() -> usize {
        1
    }
}



pub trait BlenderTransforms {
    fn scale(&mut self, factor: f32);
    fn offset(&mut self, x_offset: f32, y_offset: f32, z_offset: f32);
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct BlenderParticle {
    pub location: BlenderPoint3,
    pub prev_location: BlenderPoint3,

    pub velocity: BlenderPoint3,
    pub prev_velocity: BlenderPoint3,

    pub rotation: BlenderPoint4,
    pub prev_rotation: BlenderPoint4,
}

impl BlenderTransforms for BlenderParticle {
    fn scale(&mut self, factor: f32) {
        self.location.scale(factor);
        self.prev_location.scale(factor);

        self.velocity.scale(factor);
        self.prev_velocity.scale(factor);

        self.rotation.scale(factor);
        self.prev_rotation.scale(factor);
    }

    fn offset(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        self.location.offset(x_offset,y_offset,z_offset);
        self.prev_location.offset(x_offset,y_offset,z_offset);

        self.velocity.offset(x_offset,y_offset,z_offset);
        self.prev_velocity.offset(x_offset,y_offset,z_offset);

        self.rotation.offset(x_offset,y_offset,z_offset);
        self.prev_rotation.offset(x_offset,y_offset,z_offset);
    }
}


#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct BlenderPoint3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl BlenderTransforms for BlenderPoint3 {
    fn scale(&mut self, factor: f32) {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
    }

    fn offset(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        self.x += x_offset;
        self.y += y_offset;
        self.z += z_offset;
    }
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct BlenderPoint4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,     // not currently used/implemented
}

impl BlenderTransforms for BlenderPoint4 {
    fn scale(&mut self, factor: f32) {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self.w *= 1.0;    //ignore w term
    }

    fn offset(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        self.x += x_offset;
        self.y += y_offset;
        self.z += z_offset;
        self.w += 0.0;      //ignore w term

    }

}

impl BlenderPoint4 {
    pub fn into_bp3(self) -> BlenderPoint3
    {
        BlenderPoint3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}