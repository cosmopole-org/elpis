//! A compact software 3D renderer for `Scene3D` nodes.
//!
//! Blinc's GPU 3D path needs uploaded mesh/material resources, which the Elpis
//! bridge cannot create outside a live render context. To still give the 3D /
//! game tab *real* geometry, this module projects the declarative scene to 2D on
//! the CPU — view + perspective transform, painter's-algorithm depth sort, and
//! simple Lambert shading from the scene lights — and draws the shaded faces
//! into the same Blinc `DrawContext` the 2D canvas uses. The result is a real,
//! rotating, lit solid (driven by the guest's `onTick`), rendered with nothing
//! but `fill_path` / `stroke_path`.

use blinc_core::{Brush, Color as BColor, DrawContext, Path, Point, Stroke};

use elpis_protocol::scene3d::{Camera, Geometry, Light, Scene3DSpec, Vec3};
use elpis_protocol::style::Color;

// ---- Vector helpers -------------------------------------------------------

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mul(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn norm(a: Vec3) -> Vec3 {
    let l = dot(a, a).sqrt();
    if l > 1e-6 {
        mul(a, 1.0 / l)
    } else {
        a
    }
}

/// Apply Euler rotation (degrees, XYZ order), then scale, then translate.
fn transform_point(p: Vec3, rot_deg: Vec3, scale: Vec3, pos: Vec3) -> Vec3 {
    let mut v = [p[0] * scale[0], p[1] * scale[1], p[2] * scale[2]];
    let (rx, ry, rz) = (rot_deg[0].to_radians(), rot_deg[1].to_radians(), rot_deg[2].to_radians());
    // Rx
    let (s, c) = rx.sin_cos();
    v = [v[0], v[1] * c - v[2] * s, v[1] * s + v[2] * c];
    // Ry
    let (s, c) = ry.sin_cos();
    v = [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c];
    // Rz
    let (s, c) = rz.sin_cos();
    v = [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]];
    add(v, pos)
}

// ---- Camera ---------------------------------------------------------------

struct View {
    eye: Vec3,
    right: Vec3,
    up: Vec3,
    fwd: Vec3,
    focal: f32,
    cx: f32,
    cy: f32,
}

impl View {
    fn new(camera: &Camera, width: f32, height: f32) -> View {
        let (eye, target, fov) = match *camera {
            Camera::Perspective { position, look_at, fov, .. } => (position, look_at, fov),
            Camera::Orthographic { position, look_at, .. } => (position, look_at, 45.0),
        };
        let fwd = norm(sub(target, eye));
        let world_up = [0.0, 1.0, 0.0];
        let right = norm(cross(fwd, world_up));
        let up = cross(right, fwd);
        let focal = (height * 0.5) / (fov.to_radians() * 0.5).tan().max(1e-3);
        View { eye, right, up, fwd, focal, cx: width * 0.5, cy: height * 0.5 }
    }

    /// World point -> (screen point, camera-space depth). `None` if behind.
    fn project(&self, world: Vec3) -> Option<(Point, f32)> {
        let rel = sub(world, self.eye);
        let z = dot(rel, self.fwd);
        if z <= 0.05 {
            return None;
        }
        let x = dot(rel, self.right);
        let y = dot(rel, self.up);
        Some((Point::new(self.cx + x / z * self.focal, self.cy - y / z * self.focal), z))
    }
}

// ---- Meshes ---------------------------------------------------------------

/// A face is a loop of indices into the vertex list.
struct Mesh {
    verts: Vec<Vec3>,
    faces: Vec<Vec<usize>>,
}

fn cube(size: f32) -> Mesh {
    let h = size * 0.5;
    let verts = vec![
        [-h, -h, -h],
        [h, -h, -h],
        [h, h, -h],
        [-h, h, -h],
        [-h, -h, h],
        [h, -h, h],
        [h, h, h],
        [-h, h, h],
    ];
    let faces = vec![
        vec![0, 1, 2, 3], // back
        vec![5, 4, 7, 6], // front
        vec![4, 0, 3, 7], // left
        vec![1, 5, 6, 2], // right
        vec![3, 2, 6, 7], // top
        vec![4, 5, 1, 0], // bottom
    ];
    Mesh { verts, faces }
}

fn plane(w: f32, d: f32) -> Mesh {
    let (hw, hd) = (w * 0.5, d * 0.5);
    Mesh {
        verts: vec![[-hw, 0.0, -hd], [hw, 0.0, -hd], [hw, 0.0, hd], [-hw, 0.0, hd]],
        faces: vec![vec![0, 1, 2, 3]],
    }
}

fn uv_sphere(radius: f32, segments: u32) -> Mesh {
    let stacks = (segments / 2).max(3);
    let slices = segments.max(3);
    let mut verts = Vec::new();
    for i in 0..=stacks {
        let phi = std::f32::consts::PI * i as f32 / stacks as f32;
        let (sp, cp) = phi.sin_cos();
        for j in 0..slices {
            let theta = std::f32::consts::TAU * j as f32 / slices as f32;
            let (st, ct) = theta.sin_cos();
            verts.push([radius * sp * ct, radius * cp, radius * sp * st]);
        }
    }
    let idx = |i: u32, j: u32| (i * slices + (j % slices)) as usize;
    let mut faces = Vec::new();
    for i in 0..stacks {
        for j in 0..slices {
            faces.push(vec![
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]);
        }
    }
    Mesh { verts, faces }
}

fn mesh_for(geo: &Geometry) -> Mesh {
    match geo {
        Geometry::Cube { size } => cube(*size),
        Geometry::Plane { width, height } => plane(*width, *height),
        Geometry::Sphere { radius, segments } => uv_sphere(*radius, *segments),
        // Cylinder/cone/torus/model/custom fall back to a unit box so the
        // entity still has presence in the scene.
        Geometry::Cylinder { radius, height } => {
            let mut m = cube(radius.max(*height));
            for v in &mut m.verts {
                v[1] *= height / radius.max(0.001);
            }
            m
        }
        Geometry::Cone { radius, height } => {
            let mut m = uv_sphere(*radius, 12);
            for v in &mut m.verts {
                v[1] *= height / radius.max(0.001);
            }
            m
        }
        Geometry::Torus { radius, .. } => uv_sphere(*radius, 16),
        Geometry::Model { .. } | Geometry::Custom { .. } => cube(1.0),
    }
}

// ---- Lighting + color -----------------------------------------------------

fn bcol(c: Color, mul_: f32) -> BColor {
    BColor::rgba((c.r * mul_).clamp(0.0, 1.0), (c.g * mul_).clamp(0.0, 1.0), (c.b * mul_).clamp(0.0, 1.0), c.a)
}

struct Lighting {
    ambient: f32,
    dir: Vec3,
    dir_intensity: f32,
}

fn lighting(spec: &Scene3DSpec) -> Lighting {
    let mut ambient = 0.25;
    let mut dir = norm([-0.4, -1.0, -0.6]);
    let mut dir_intensity = 0.85;
    for l in &spec.lights {
        match *l {
            Light::Ambient { intensity, .. } => ambient = intensity,
            Light::Directional { direction, intensity, .. } => {
                dir = norm(direction);
                dir_intensity = intensity;
            }
            _ => {}
        }
    }
    Lighting { ambient, dir, dir_intensity }
}

// ---- Render ---------------------------------------------------------------

struct DrawFace {
    pts: Vec<Point>,
    depth: f32,
    color: BColor,
}

/// Render the scene into `ctx`, filling the `width` x `height` canvas region.
pub fn render(ctx: &mut dyn DrawContext, width: f32, height: f32, spec: &Scene3DSpec) {
    // Background.
    let bg = spec.background.map(|c| bcol(c, 1.0)).unwrap_or(BColor::rgba(0.05, 0.06, 0.09, 1.0));
    ctx.fill_rect(
        blinc_core::Rect::new(0.0, 0.0, width, height),
        blinc_core::CornerRadius::ZERO,
        Brush::Solid(bg),
    );

    let view = View::new(&spec.camera, width, height);
    let light = lighting(spec);

    let mut faces: Vec<DrawFace> = Vec::new();
    collect_entities(&spec.entities, &view, &light, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], &mut faces);

    // Painter's algorithm: far faces first.
    faces.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

    let edge = Stroke::new(1.0);
    for f in faces {
        if f.pts.len() < 3 {
            continue;
        }
        let mut path = Path::new().move_to(f.pts[0].x, f.pts[0].y);
        for p in &f.pts[1..] {
            path = path.line_to(p.x, p.y);
        }
        path = path.close();
        ctx.fill_path(&path, Brush::Solid(f.color));
        ctx.stroke_path(&path, &edge, Brush::Solid(BColor::rgba(0.0, 0.0, 0.0, 0.25)));
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_entities(
    entities: &[elpis_protocol::scene3d::Entity],
    view: &View,
    light: &Lighting,
    parent_pos: Vec3,
    parent_rot: Vec3,
    parent_scale: Vec3,
    out: &mut Vec<DrawFace>,
) {
    for e in entities {
        let pos = add(parent_pos, e.transform.position);
        let rot = add(parent_rot, e.transform.rotation);
        let scale = [
            parent_scale[0] * e.transform.scale[0],
            parent_scale[1] * e.transform.scale[1],
            parent_scale[2] * e.transform.scale[2],
        ];

        if let Some(geo) = &e.geometry {
            let mesh = mesh_for(geo);
            let base = e.material.as_ref().map(|m| m.base_color).unwrap_or(Color::rgb(0.7, 0.7, 0.75));
            // World-space vertices.
            let world: Vec<Vec3> =
                mesh.verts.iter().map(|v| transform_point(*v, rot, scale, pos)).collect();

            for face in &mesh.faces {
                // Project all face vertices; skip the face if any is behind.
                let mut pts = Vec::with_capacity(face.len());
                let mut depth = 0.0f32;
                let mut ok = true;
                for &i in face {
                    match view.project(world[i]) {
                        Some((p, z)) => {
                            pts.push(p);
                            depth += z;
                        }
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok || face.len() < 3 {
                    continue;
                }
                depth /= face.len() as f32;

                // Flat shading from the face's world normal.
                let n = norm(cross(
                    sub(world[face[1]], world[face[0]]),
                    sub(world[face[2]], world[face[0]]),
                ));
                let diffuse = dot(n, mul(light.dir, -1.0)).max(0.0) * light.dir_intensity;
                let shade = (light.ambient + diffuse).clamp(0.0, 1.2);
                out.push(DrawFace { pts, depth, color: bcol(base, shade) });
            }
        }

        if !e.children.is_empty() {
            collect_entities(&e.children, view, light, pos, rot, scale, out);
        }
    }
}
