use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions, MouseButton, MouseMode};
use std::time::Duration;

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod skybox;
mod fragment;
mod shaders;

use nalgebra_glm as glm;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use shaders::vertex_shader;
use color::Color;
use skybox::Skybox;

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub light_dir: Vec3,
    pub base_color: Color,
    pub ambient: f32,
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    edge_overlay: bool,
    edge_color: Color,
) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut tris = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            tris.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization + Fragment Collection
    let mut fragments = Vec::new();
    for tri in &tris {
        fragments.extend(crate::triangle::triangle(
            &tri[0],
            &tri[1],
            &tri[2],
            uniforms.light_dir,
            uniforms.base_color,
            uniforms.ambient,
        ));
    }

    // Fragment Processing Stage (per-fragment color + depth test)
    for frag in fragments {
        let x = frag.position.x as isize;
        let y = frag.position.y as isize;
        if x >= 0 && y >= 0
            && (x as usize) < framebuffer.width
            && (y as usize) < framebuffer.height
        {
            framebuffer.plot(x as usize, y as usize, frag.color.to_hex(), frag.depth);
        }
    }

    // Optional: draw dark panel edges on top (depth-tested)
    if edge_overlay {
        for tri in &tris {
            let mut edges = Vec::new();
            edges.extend(crate::line::line(&tri[0], &tri[1]));
            edges.extend(crate::line::line(&tri[1], &tri[2]));
            edges.extend(crate::line::line(&tri[2], &tri[0]));
            for f in edges {
                let x = f.position.x as isize;
                let y = f.position.y as isize;
                if x >= 0 && y >= 0
                    && (x as usize) < framebuffer.width
                    && (y as usize) < framebuffer.height
                {
                    framebuffer.plot(x as usize, y as usize, edge_color.to_hex(), f.depth);
                }
            }
        }
    }
}

fn main() {
    let window_width = 1280;
    let window_height = 720;
    let framebuffer_width = 1280;
    let framebuffer_height = 720;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Spaceship",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

  let hull_color = Color::from_hex(0x2E3A58); // Deep navy
  let edge_color = Color::from_hex(0x1A2030); // Copper-ish dark for panel lines
  let mut edge_overlay = true;                // keep edge overlay support

    let skybox = Skybox::load("assets/skybox");
let fov_y = 60.0_f32.to_radians();
let mut yaw: f32 = 0.0;
let mut pitch: f32 = 0.0;

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    let mut translation = Vec3::new((framebuffer_width as f32) * 0.5, (framebuffer_height as f32) * 0.5, 0.0);
    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 100.0f32;

    let obj = Obj::load("assets/models/spaceship.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array(); 

      // Mouse drag rotation state
      let mut last_mouse_pos: Option<(f32, f32)> = None;
      let mouse_sensitivity: f32 = 0.01;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        // Edge overlay toggle (O=on, P=off)
        if window.is_key_down(Key::O) { edge_overlay = true; }
        if window.is_key_down(Key::P) { edge_overlay = false; }

          // Mouse drag (left button) controls rotation; pinch/drag gesture on trackpads maps to this
          if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
              if window.get_mouse_down(MouseButton::Left) {
                  if let Some((pmx, pmy)) = last_mouse_pos {
                      let dx = mx - pmx;
                      let dy = my - pmy;
                      // Yaw around Y with horizontal drag, pitch around X with vertical drag
                      rotation.y += dx as f32 * mouse_sensitivity;
                      rotation.x += dy as f32 * mouse_sensitivity;
                  }
                  last_mouse_pos = Some((mx, my));
              } else {
                  // Update without rotating so first click doesn't jump
                  last_mouse_pos = Some((mx, my));
              }
          }

        handle_input(&window, &mut translation, &mut rotation, &mut scale);

        framebuffer.clear();

        let model_matrix = create_model_matrix(translation, scale, rotation);
        let light_dir = glm::normalize(&Vec3::new(-0.4, -0.7, -1.0));
        let ambient = 0.2;

        let base_color = hull_color;
        let uniforms = Uniforms { model_matrix, light_dir, base_color, ambient };

        // Draw skybox then model
        skybox.draw(&mut framebuffer, fov_y, yaw, pitch);
        render(&mut framebuffer, &uniforms, &vertex_arrays, edge_overlay, edge_color);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(window: &Window, _translation: &mut Vec3, _rotation: &mut Vec3, scale: &mut f32) {
    if window.is_key_down(Key::S) {
        *scale *= 1.02;
    }
    if window.is_key_down(Key::A) {
        *scale *= 0.98;
    }
}
