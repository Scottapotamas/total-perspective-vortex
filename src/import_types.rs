use colorsys::Hsl;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum BlenderData {
    #[serde(rename = "poly")]
    PolySpline(BlenderPoly),
    NURBSSpline(BlenderNURBS),
    Particles(BlenderParticles),
}

pub trait Spline {

    fn close_loop(&mut self) {
        // optional - particle systems don't have loops
    }

    fn scale_points(&mut self, factor: f32);
    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32);

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
        self.points.iter().for_each(|mut p| p.scale(factor) );
        self.curve_length *= factor;
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        for mut point in self.points {
            point.offset(x_offset, y_offset, z_offset);
        }
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
            self.points.push(self.points[0].clone());
            self.points.push(self.points[1].clone());
            self.points.push(self.points[2].clone());

        }
    }

    fn scale_points(&mut self, factor: f32) {
        self.points.iter().for_each(|mut p| p.scale(factor) );
        self.curve_length *= factor;
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32) {
        for mut point in self.points {
            point.offset(x_offset, y_offset, z_offset);
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlenderParticles {
    pub particles: Vec<BlenderParticle>,
    pub color_rgba: (f32,f32,f32,f32),

    #[serde(skip)]
    pub color: Vec<Hsl>,
}

impl Spline for BlenderParticles {

    fn close_loop(&mut self) {
        unimplemented!();
    }

    fn scale_points(&mut self, factor: f32) {
        self.particles.iter().for_each(|mut p| p.scale(factor) );
    }

    fn offset_points(&mut self, x_offset: f32, y_offset: f32, z_offset: f32 ) {
        for mut particle in self.particles {
            particle.offset(x_offset,y_offset,z_offset);
        }
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
