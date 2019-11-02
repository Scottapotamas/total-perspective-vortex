use colorsys::Hsl;
use colorsys::Rgb;

// Calculates the distance between two HSL values as expected by human vision models
// Its OK to add a constant k to the lightness difference as it has arbitrary importance
pub fn distance_hsl(x: &Hsl, y: &Hsl) -> f64 {
    // HSL must be [0, 2pi), [0, 1], [0, 1] first
    let h1 = x.get_hue().to_radians();
    let s1 = x.get_saturation();
    let l1 = x.get_lightness();

    let h2 = y.get_hue().to_radians();
    let s2 = y.get_saturation();
    let l2 = y.get_lightness();

    // Project the colors into linear space (a,b,c)
    let a1 = h1.cos() * s1 * l1;
    let b1 = h1.sin() * s1 * l1;
    let c1 = l1;

    let a2 = h2.cos() * s2 * l2;
    let b2 = h2.sin() * s2 * l2;
    let c2 = l2;

    // Simple cartesian distance
    let d2 = (a1 - a2).powi(2) + (b1 - b2).powi(2) + (c1 - c2).powi(2);
    d2.sqrt()
}

pub fn delta_led_from_hsl(color: &Hsl) -> (f32, f32, f32) {
    return (
        color.get_hue() as f32 / 360.0,
        color.get_saturation() as f32 / 100.0,
        color.get_lightness() as f32 / 100.0,
    );
}

pub fn hsl_to_rgb8(color: &Hsl) -> (u8, u8, u8) {
    let rgb = Rgb::from(color);

    return (
        rgb.get_red() as u8,
        rgb.get_green() as u8,
        rgb.get_blue() as u8,
    );
}
