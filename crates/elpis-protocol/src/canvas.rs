//! Immediate-mode 2D drawing ops for a `Canvas` node.
//!
//! A Miniapp builds a `Canvas` by emitting a flat list of [`DrawOp`]s, which the
//! Blinc backend replays into a `DrawContext` (Blinc's GPU 2D drawing surface,
//! the same one its `canvas` element and paint crate expose). This covers the
//! full Blinc paint vocabulary: filled/stroked rects, rounded rects, circles,
//! ellipses, lines, polylines, arbitrary bezier paths, arcs, gradients, text,
//! images, clipping, and the transform/layer stack.

use serde::{Deserialize, Serialize};

use crate::style::{Brush, Color};

/// A 2D point in canvas space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

/// A rectangle (top-left origin, width/height).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// How a stroked line terminates / joins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// One segment of a vector path (SVG-style mini language).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PathSeg {
    MoveTo { x: f32, y: f32 },
    LineTo { x: f32, y: f32 },
    QuadTo { cx: f32, cy: f32, x: f32, y: f32 },
    CubicTo { c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32 },
    ArcTo { rx: f32, ry: f32, rotation: f32, large: bool, sweep: bool, x: f32, y: f32 },
    Close,
}

/// Stroke parameters shared by stroking ops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub brush: Brush,
    #[serde(default = "default_width")]
    pub width: f32,
    #[serde(default)]
    pub cap: LineCap,
    #[serde(default)]
    pub join: LineJoin,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dash: Vec<f32>,
}

fn default_width() -> f32 {
    1.0
}

/// A single immediate-mode drawing instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DrawOp {
    /// Clear the whole canvas to a color.
    Clear { color: Color },
    FillRect { rect: Rect, brush: Brush },
    StrokeRect { rect: Rect, stroke: Stroke },
    FillRoundRect { rect: Rect, radius: f32, brush: Brush },
    StrokeRoundRect { rect: Rect, radius: f32, stroke: Stroke },
    FillCircle { center: Point, radius: f32, brush: Brush },
    StrokeCircle { center: Point, radius: f32, stroke: Stroke },
    FillEllipse { center: Point, rx: f32, ry: f32, brush: Brush },
    Line { from: Point, to: Point, stroke: Stroke },
    Polyline { points: Vec<Point>, stroke: Stroke },
    Polygon { points: Vec<Point>, brush: Brush },
    /// Arc / pie / ring sector.
    Arc {
        center: Point,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        brush: Option<Brush>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stroke: Option<Stroke>,
    },
    /// Fill an arbitrary path.
    FillPath { segments: Vec<PathSeg>, brush: Brush },
    /// Stroke an arbitrary path.
    StrokePath { segments: Vec<PathSeg>, stroke: Stroke },
    /// Draw text at a baseline point.
    Text {
        text: String,
        at: Point,
        #[serde(default = "default_font_size")]
        size: f32,
        color: Color,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        font: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align: Option<String>,
    },
    /// Blit an image (by asset id / URL) into a destination rect.
    Image { src: String, dest: Rect },
    // ---- State stack ---------------------------------------------------
    Save,
    Restore,
    Translate { x: f32, y: f32 },
    Scale { x: f32, y: f32 },
    Rotate { degrees: f32 },
    /// Push a rectangular clip.
    ClipRect { rect: Rect },
    /// Push a path clip.
    ClipPath { segments: Vec<PathSeg> },
    /// Set global layer opacity for subsequent ops.
    GlobalAlpha { alpha: f32 },
}

fn default_font_size() -> f32 {
    14.0
}

/// The payload of a `Canvas` node.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CanvasSpec {
    #[serde(default)]
    pub ops: Vec<DrawOp>,
    /// Whether the canvas re-runs the guest's `paint` callback every frame
    /// (animated) versus only when its op list changes (static).
    #[serde(default)]
    pub animated: bool,
}
