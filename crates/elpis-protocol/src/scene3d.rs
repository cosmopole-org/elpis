//! 3D scene + game description for a `Scene3D` node.
//!
//! Blinc's ecosystem includes 3D ("3fld") rendering and game-making features —
//! meshes, materials, lights, cameras, sprites, and a per-frame update tick. A
//! Miniapp describes a scene declaratively here; the backend maps it to Blinc's
//! 3D pipeline and drives the guest's `tick` handler each frame for game logic.

use serde::{Deserialize, Serialize};

use crate::style::Color;

/// A 3D vector / position / euler-rotation triple.
pub type Vec3 = [f32; 3];

/// A node transform in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3D {
    #[serde(default)]
    pub position: Vec3,
    /// Euler rotation in degrees (xyz).
    #[serde(default)]
    pub rotation: Vec3,
    #[serde(default = "unit_scale")]
    pub scale: Vec3,
}

fn unit_scale() -> Vec3 {
    [1.0, 1.0, 1.0]
}

impl Default for Transform3D {
    fn default() -> Self {
        Transform3D { position: [0.0; 3], rotation: [0.0; 3], scale: [1.0; 3] }
    }
}

/// Camera describing the view.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Camera {
    Perspective {
        position: Vec3,
        look_at: Vec3,
        #[serde(default = "default_fov")]
        fov: f32,
        #[serde(default = "default_near")]
        near: f32,
        #[serde(default = "default_far")]
        far: f32,
    },
    Orthographic {
        position: Vec3,
        look_at: Vec3,
        #[serde(default = "one_f")]
        scale: f32,
    },
}

fn default_fov() -> f32 {
    60.0
}
fn default_near() -> f32 {
    0.1
}
fn default_far() -> f32 {
    1000.0
}
fn one_f() -> f32 {
    1.0
}

impl Default for Camera {
    fn default() -> Self {
        Camera::Perspective {
            position: [0.0, 0.0, 5.0],
            look_at: [0.0, 0.0, 0.0],
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// A scene light.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Light {
    Ambient { color: Color, intensity: f32 },
    Directional { direction: Vec3, color: Color, intensity: f32 },
    Point { position: Vec3, color: Color, intensity: f32, #[serde(default)] range: f32 },
    Spot { position: Vec3, direction: Vec3, color: Color, intensity: f32, #[serde(default)] angle: f32 },
}

/// A PBR-ish material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    #[serde(default = "white")]
    pub base_color: Color,
    #[serde(default)]
    pub metallic: f32,
    #[serde(default = "half")]
    pub roughness: f32,
    #[serde(default)]
    pub emissive: Option<Color>,
    /// Texture asset id / URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    /// Normal-map asset id / URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal_map: Option<String>,
    #[serde(default = "one_f")]
    pub opacity: f32,
}

fn white() -> Color {
    Color::WHITE
}
fn half() -> f32 {
    0.5
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Color::WHITE,
            metallic: 0.0,
            roughness: 0.5,
            emissive: None,
            texture: None,
            normal_map: None,
            opacity: 1.0,
        }
    }
}

/// Geometry source for a mesh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Geometry {
    Cube { #[serde(default = "one_f")] size: f32 },
    Sphere { #[serde(default = "one_f")] radius: f32, #[serde(default = "default_segments")] segments: u32 },
    Plane { #[serde(default = "one_f")] width: f32, #[serde(default = "one_f")] height: f32 },
    Cylinder { #[serde(default = "one_f")] radius: f32, #[serde(default = "one_f")] height: f32 },
    Cone { #[serde(default = "one_f")] radius: f32, #[serde(default = "one_f")] height: f32 },
    Torus { #[serde(default = "one_f")] radius: f32, #[serde(default = "default_tube")] tube: f32 },
    /// Load an external model (glTF / OBJ) by asset id / URL.
    Model { src: String },
    /// Raw mesh data.
    Custom { vertices: Vec<f32>, indices: Vec<u32>, #[serde(default)] normals: Vec<f32>, #[serde(default)] uvs: Vec<f32> },
}

fn default_segments() -> u32 {
    32
}
fn default_tube() -> f32 {
    0.3
}

/// A 2D sprite billboarded in the scene (game making).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    pub texture: String,
    #[serde(default)]
    pub position: Vec3,
    #[serde(default = "one_f")]
    pub scale: f32,
    /// Optional source rect within a sprite atlas `[x, y, w, h]` in UV space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<[f32; 4]>,
}

/// One node in the scene graph. An entity may carry a mesh and/or sprite plus
/// children, and is identified by `id` so the guest's game logic can address it
/// and the differ can reconcile it across frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default)]
    pub transform: Transform3D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Geometry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<Sprite>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Entity>,
    /// Whether the entity is interactive (raycast/pick → emits `pick` events).
    #[serde(default)]
    pub pickable: bool,
}

/// The payload of a `Scene3D` node.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Scene3DSpec {
    #[serde(default)]
    pub camera: Camera,
    #[serde(default)]
    pub lights: Vec<Light>,
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Color>,
    /// Skybox / environment map asset id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Whether the scene drives a per-frame game `tick` in the guest.
    #[serde(default)]
    pub animated: bool,
    /// Enable simple physics integration on entities (gravity/velocity).
    #[serde(default)]
    pub physics: bool,
}
