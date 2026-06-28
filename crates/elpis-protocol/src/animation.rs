//! Animations attached to a node.
//!
//! Blinc's animation system is built on spring physics, keyframe timelines, and
//! orchestrated transitions (the `blinc_animation` crate). A Miniapp attaches
//! one or more [`Animation`]s to a node; the backend installs them on the
//! corresponding Blinc element and reports completion back to the guest via an
//! event when requested.

use serde::{Deserialize, Serialize};

/// An animatable property of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimProp {
    Opacity,
    TranslateX,
    TranslateY,
    Scale,
    ScaleX,
    ScaleY,
    Rotate,
    RotateX,
    RotateY,
    Width,
    Height,
    BackgroundColor,
    ForegroundColor,
    BorderRadius,
    Blur,
    /// Animate the element's measured layout bounds (Blinc `animate_layout`).
    Layout,
}

/// Spring physics parameters (Blinc `SpringConfig`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    #[serde(default = "default_stiffness")]
    pub stiffness: f32,
    #[serde(default = "default_damping")]
    pub damping: f32,
    #[serde(default = "one_f")]
    pub mass: f32,
    #[serde(default)]
    pub initial_velocity: f32,
}

fn default_stiffness() -> f32 {
    170.0
}
fn default_damping() -> f32 {
    26.0
}
fn one_f() -> f32 {
    1.0
}

impl Default for Spring {
    fn default() -> Self {
        Spring { stiffness: 170.0, damping: 26.0, mass: 1.0, initial_velocity: 0.0 }
    }
}

/// A standard easing curve for tween-based animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    #[default]
    EaseInOut,
    /// A cubic-bezier, parameters carried separately in [`Curve`].
    CubicBezier,
}

/// The driver of an animation: spring physics or a timed tween.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Curve {
    Spring(Spring),
    Tween {
        /// Duration in milliseconds.
        duration: f32,
        #[serde(default)]
        easing: Easing,
        /// Bezier control points when `easing == cubic_bezier`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bezier: Option<[f32; 4]>,
    },
}

impl Default for Curve {
    fn default() -> Self {
        Curve::Spring(Spring::default())
    }
}

/// One keyframe in a multi-stop timeline: a value at a normalized time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Normalized time `0.0..=1.0`.
    pub at: f32,
    pub value: f32,
}

/// How an animation repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Repeat {
    #[default]
    Once,
    Loop,
    PingPong,
}

/// A single animation on a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation {
    pub prop: AnimProp,
    /// Target value (for a simple from→to animation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<f32>,
    /// Optional explicit starting value (defaults to the current value).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<f32>,
    /// Multi-stop timeline (overrides `from`/`to` when present).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframes: Vec<Keyframe>,
    #[serde(default)]
    pub curve: Curve,
    #[serde(default)]
    pub repeat: Repeat,
    /// Start delay in milliseconds.
    #[serde(default)]
    pub delay: f32,
    /// Event name fired at the guest when the animation completes (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<String>,
}

/// A layout transition installed on a node so that subsequent prop changes are
/// animated rather than snapped (Blinc `animate_layout` / motion).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Transition {
    #[serde(default)]
    pub curve: Curve,
    #[serde(default)]
    pub enabled: bool,
}
