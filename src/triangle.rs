use nalgebra_glm as glm;
use nalgebra_glm::Vec3;

use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;

// Signed area helper (edge function)
#[inline]
fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

pub fn triangle(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    light_dir: Vec3,   // unit vector, direction FROM light to the scene
    base: Color,       // base color for the ship (e.g., light gray/blue)
    ambient: f32,      // 0..1
) -> Vec<Fragment> {
    let p0 = v0.transformed_position;
    let p1 = v1.transformed_position;
    let p2 = v2.transformed_position;

    // Degenerate? (area ~ 0) → nothing to draw
    let area = edge(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y);
    if area.abs() < f32::EPSILON {
        return Vec::new();
    }

    // Compute face normal in 3D from positions. We don't have a camera yet, so
    // this is "model/screen space" normal—good enough to reveal facets.
    let e1 = p1 - p0;
    let e2 = p2 - p0;
    let mut n = glm::cross(&e1, &e2);
    if glm::length(&n) > 0.0 {
        n = glm::normalize(&n);
    }

    // Lambert: intensity = ambient + max(0, N · (-L))
    let l = -light_dir; // convert "from light to scene" into "towards light"
    let diff = 0.0f32.max(glm::dot(&n, &l));
    let intensity = (ambient + diff).clamp(0.0, 1.0);

    // Convert base color to floats, scale by intensity
    let (br, bg, bb) = (
        (base.to_hex() >> 16) as u8 as f32 / 255.0,
        ((base.to_hex() >> 8)  & 0xFF) as u8 as f32 / 255.0,
        (base.to_hex() & 0xFF) as u8 as f32 / 255.0,
    );
    let shaded = Color::from_float(br * intensity, bg * intensity, bb * intensity);

    // Bounding box (in screen space)
    let min_x = p0.x.min(p1.x).min(p2.x).floor() as i32;
    let max_x = p0.x.max(p1.x).max(p2.x).ceil() as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor() as i32;
    let max_y = p0.y.max(p1.y).max(p2.y).ceil() as i32;

    let mut frags = Vec::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let w0 = edge(p1.x, p1.y, p2.x, p2.y, px, py);
            let w1 = edge(p2.x, p2.y, p0.x, p0.y, px, py);
            let w2 = edge(p0.x, p0.y, p1.x, p1.y, px, py);

            let inside = if area > 0.0 {
                w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
            } else {
                w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
            };

            if inside {
                // Interpolate z for depth test
                let inv_area = 1.0 / area;
                let b0 = w0 * inv_area;
                let b1 = w1 * inv_area;
                let b2 = w2 * inv_area;
                let z = b0 * p0.z + b1 * p1.z + b2 * p2.z;

                frags.push(Fragment::new(px, py, shaded, z));
            }
        }
    }

    frags
}