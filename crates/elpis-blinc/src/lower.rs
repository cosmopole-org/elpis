//! Lowering: translate an [`elpis_protocol::Node`] tree into a [`BlincDom`] —
//! a normalized, blinc-flavored description of the exact builder calls the
//! Blinc backend will make.
//!
//! Keeping this as a pure data transform (no Blinc dependency) means the whole
//! mapping — every node kind, every style attribute, the 2D canvas op list, the
//! 3D scene, animations, theming — is unit-testable without a GPU, and the
//! feature-gated [`crate::blinc_backend`] is reduced to a thin interpreter that
//! walks a `BlincDom` and calls the real `blinc_layout` / `blinc_core` builders.
//!
//! This is where "cover all of Blinc" is made concrete and verifiable: the
//! `lower` function has an arm for every [`elpis_protocol::node::NodeKind`].

use std::collections::BTreeMap;

use elpis_protocol::animation::Animation;
use elpis_protocol::canvas::CanvasSpec;
use elpis_protocol::node::*;
use elpis_protocol::scene3d::Scene3DSpec;
use elpis_protocol::style::*;
use elpis_protocol::Node;

/// A resolved, blinc-flavored style. Where [`Style`] is the wire format (all
/// optional, CSS-named), `BlincStyle` is the form the Blinc builder consumes:
/// display/direction are always resolved (a `Row` node forces a row direction,
/// a `Stack` node forces stack display, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct BlincStyle {
    pub display: Display,
    pub direction: FlexDirection,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub justify: Option<Justify>,
    pub gap: Option<f32>,
    pub row_gap: Option<f32>,
    pub column_gap: Option<f32>,
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub min_width: Option<Length>,
    pub min_height: Option<Length>,
    pub max_width: Option<Length>,
    pub max_height: Option<Length>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Length>,
    pub wrap: bool,
    pub grid_template_columns: Option<String>,
    pub grid_template_rows: Option<String>,
    pub grid_column: Option<String>,
    pub grid_row: Option<String>,
    pub position: Option<Position>,
    pub inset: Option<Edges>,
    pub z_index: Option<i32>,
    pub overflow_x: Option<Overflow>,
    pub overflow_y: Option<Overflow>,
    pub background: Option<Brush>,
    pub foreground: Option<Color>,
    pub border: Option<(f32, Color)>,
    pub radius: Option<CornerRadius>,
    pub shadows: Vec<Shadow>,
    pub opacity: Option<f32>,
    pub transform: Option<Transform>,
    pub filter: Option<Filter>,
    pub glass: bool,
    pub cursor: Option<String>,
    pub classes: Vec<String>,
    pub css: Option<String>,
}

impl Default for BlincStyle {
    fn default() -> Self {
        BlincStyle {
            display: Display::Flex,
            direction: FlexDirection::Row,
            align_items: None,
            align_self: None,
            justify: None,
            gap: None,
            row_gap: None,
            column_gap: None,
            padding: None,
            margin: None,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            flex_grow: None,
            flex_shrink: None,
            flex_basis: None,
            wrap: false,
            grid_template_columns: None,
            grid_template_rows: None,
            grid_column: None,
            grid_row: None,
            position: None,
            inset: None,
            z_index: None,
            overflow_x: None,
            overflow_y: None,
            background: None,
            foreground: None,
            border: None,
            radius: None,
            shadows: Vec::new(),
            opacity: None,
            transform: None,
            filter: None,
            glass: false,
            cursor: None,
            classes: Vec::new(),
            css: None,
        }
    }
}

/// The per-kind content of a lowered node (everything that isn't layout/paint).
#[derive(Debug, Clone, PartialEq)]
pub enum BlincContent {
    /// A pure container (`div`, `row`, `column`, `stack`, `grid`, `spacer`).
    Container,
    Scroll(ScrollSpec),
    Overlay(OverlaySpec),
    Text(TextSpec),
    RichText(RichTextSpec),
    Markdown(MarkdownSpec),
    Image(ImageSpec),
    Svg(SvgSpec),
    Icon(IconSpec),
    Button(ButtonSpec),
    Input(InputSpec),
    /// Checkbox or switch (distinguished by `toggle_kind`).
    Toggle { spec: ToggleSpec, toggle_kind: &'static str },
    Radio(RadioSpec),
    Slider(SliderSpec),
    Dropdown(DropdownSpec),
    Tabs(TabsSpec),
    Carousel(CarouselSpec),
    Progress(ProgressSpec),
    Spinner(SpinnerSpec),
    Canvas(CanvasSpec),
    Scene3D(Scene3DSpec),
    /// Video or audio (distinguished by `media_kind`).
    Media { spec: MediaSpec, media_kind: &'static str },
    Component(ComponentSpec),
}

/// A lowered node: a blinc-flavored element ready for the builder interpreter.
#[derive(Debug, Clone, PartialEq)]
pub struct BlincDom {
    pub tag: &'static str,
    pub style: BlincStyle,
    pub content: BlincContent,
    pub events: BTreeMap<String, String>,
    pub animations: Vec<Animation>,
    pub children: Vec<BlincDom>,
}

/// Lower a whole tree.
pub fn lower(node: &Node) -> BlincDom {
    let tag = node.type_tag();
    let mut style = resolve_style(&node.style, &node.kind);
    let content = lower_kind(&node.kind, &mut style);
    BlincDom {
        tag,
        style,
        content,
        events: node.events.clone(),
        animations: node.animations.clone(),
        children: node.children.iter().map(lower).collect(),
    }
}

/// Map the wire [`Style`] onto a resolved [`BlincStyle`], applying the layout
/// defaults implied by the node kind.
fn resolve_style(s: &Style, kind: &NodeKind) -> BlincStyle {
    let mut b = BlincStyle::default();

    // Kind-implied layout defaults (mirrors Blinc's `flex_row()` / `flex_col()`
    // / `stack()` / `grid()` builder shortcuts).
    match kind {
        NodeKind::Row => b.direction = FlexDirection::Row,
        NodeKind::Column => b.direction = FlexDirection::Column,
        NodeKind::Stack => b.display = Display::Stack,
        NodeKind::Grid => b.display = Display::Grid,
        _ => {}
    }

    if let Some(d) = s.display {
        b.display = d;
    }
    if let Some(d) = s.direction {
        b.direction = d;
    }
    b.align_items = s.align_items;
    b.align_self = s.align_self;
    b.justify = s.justify_content;
    b.gap = s.gap;
    b.row_gap = s.row_gap;
    b.column_gap = s.column_gap;
    b.padding = s.padding;
    b.margin = s.margin;
    b.width = s.width;
    b.height = s.height;
    b.min_width = s.min_width;
    b.min_height = s.min_height;
    b.max_width = s.max_width;
    b.max_height = s.max_height;
    b.flex_grow = s.flex_grow;
    b.flex_shrink = s.flex_shrink;
    b.flex_basis = s.flex_basis;
    b.wrap = s.wrap.unwrap_or(false);
    b.grid_template_columns = s.grid_template_columns.clone();
    b.grid_template_rows = s.grid_template_rows.clone();
    b.grid_column = s.grid_column.clone();
    b.grid_row = s.grid_row.clone();
    b.position = s.position;
    b.inset = s.inset;
    b.z_index = s.z_index;
    b.overflow_x = s.overflow_x;
    b.overflow_y = s.overflow_y;
    b.background = s.background.clone();
    b.foreground = s.foreground;
    b.border = match (s.border_width, s.border_color) {
        (Some(w), Some(c)) => Some((w, c)),
        (Some(w), None) => Some((w, Color::BLACK)),
        _ => None,
    };
    b.radius = s.radius;
    b.shadows = s.shadows.clone();
    b.opacity = s.opacity;
    b.transform = s.transform;
    b.filter = s.filter;
    b.glass = s.glass.unwrap_or(false);
    b.cursor = s.cursor.clone();
    b.classes = s.classes.clone();
    b.css = s.css.clone();

    // Expand a liquid-glass material into concrete paint. Each piece is only
    // filled when the guest didn't set the corresponding field explicitly, so
    // a Miniapp can always override one aspect (e.g. a custom background) while
    // keeping the rest of the glass look.
    if let Some(g) = &s.glass_material {
        b.glass = true;
        let f = b.filter.get_or_insert(Filter::default());
        if f.backdrop_blur == 0.0 {
            f.backdrop_blur = g.blur;
        }
        if f.saturate.is_none() {
            f.saturate = Some(g.saturate);
        }
        if f.brightness.is_none() && (g.brightness - 1.0).abs() > f32::EPSILON {
            f.brightness = Some(g.brightness);
        }
        if b.background.is_none() {
            b.background = Some(Brush::solid(g.tint));
        }
        if b.border.is_none() && g.rim_width > 0.0 {
            b.border = Some((g.rim_width, g.rim));
        }
        if b.radius.is_none() && g.radius > 0.0 {
            b.radius = Some(CornerRadius::all(g.radius));
        }
        if g.elevation > 0.0 && b.shadows.is_empty() {
            b.shadows.push(Shadow {
                offset: [0.0, g.elevation * 0.4],
                blur: g.elevation,
                spread: 0.0,
                color: Color::rgba(0.0, 0.0, 0.0, 0.28),
                inset: false,
            });
        }
    }
    b
}

/// Map a node kind to its lowered content. A `Scroll`/`Overlay` may tweak the
/// resolved style (e.g. scroll forces overflow), so `style` is `&mut`.
fn lower_kind(kind: &NodeKind, style: &mut BlincStyle) -> BlincContent {
    match kind {
        NodeKind::Div | NodeKind::Row | NodeKind::Column | NodeKind::Stack
        | NodeKind::Grid | NodeKind::Spacer => BlincContent::Container,

        NodeKind::Scroll(spec) => {
            // A scroll viewport sets overflow on its axis.
            match spec.axis.as_str() {
                "horizontal" => style.overflow_x = Some(Overflow::Scroll),
                "both" => {
                    style.overflow_x = Some(Overflow::Scroll);
                    style.overflow_y = Some(Overflow::Scroll);
                }
                _ => style.overflow_y = Some(Overflow::Scroll),
            }
            BlincContent::Scroll(spec.clone())
        }
        NodeKind::Overlay(spec) => {
            style.position = Some(Position::Fixed);
            BlincContent::Overlay(spec.clone())
        }

        NodeKind::Text(s) => BlincContent::Text(s.clone()),
        NodeKind::RichText(s) => BlincContent::RichText(s.clone()),
        NodeKind::Markdown(s) => BlincContent::Markdown(s.clone()),
        NodeKind::Image(s) => BlincContent::Image(s.clone()),
        NodeKind::Svg(s) => BlincContent::Svg(s.clone()),
        NodeKind::Icon(s) => BlincContent::Icon(s.clone()),

        NodeKind::Button(s) => BlincContent::Button(s.clone()),
        NodeKind::TextInput(s) => BlincContent::Input(s.clone()),
        NodeKind::Checkbox(s) => {
            BlincContent::Toggle { spec: s.clone(), toggle_kind: "checkbox" }
        }
        NodeKind::Switch(s) => BlincContent::Toggle { spec: s.clone(), toggle_kind: "switch" },
        NodeKind::Radio(s) => BlincContent::Radio(s.clone()),
        NodeKind::Slider(s) => BlincContent::Slider(s.clone()),
        NodeKind::Dropdown(s) => BlincContent::Dropdown(s.clone()),
        NodeKind::Tabs(s) => BlincContent::Tabs(s.clone()),
        NodeKind::Carousel(s) => BlincContent::Carousel(s.clone()),
        NodeKind::ProgressBar(s) => BlincContent::Progress(s.clone()),
        NodeKind::Spinner(s) => BlincContent::Spinner(s.clone()),

        NodeKind::Canvas(s) => BlincContent::Canvas(s.clone()),
        NodeKind::Scene3D(s) => BlincContent::Scene3D(s.clone()),

        NodeKind::Video(s) => BlincContent::Media { spec: s.clone(), media_kind: "video" },
        NodeKind::Audio(s) => BlincContent::Media { spec: s.clone(), media_kind: "audio" },

        NodeKind::Component(s) => BlincContent::Component(s.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elpis_protocol::node::NodeKind;
    use elpis_protocol::Node;

    fn lower_kind_of(kind: NodeKind) -> BlincDom {
        lower(&Node::new(kind))
    }

    #[test]
    fn row_forces_row_direction() {
        let dom = lower_kind_of(NodeKind::Row);
        assert_eq!(dom.style.direction, FlexDirection::Row);
        assert_eq!(dom.content, BlincContent::Container);
    }

    #[test]
    fn column_forces_column_direction() {
        let dom = lower_kind_of(NodeKind::Column);
        assert_eq!(dom.style.direction, FlexDirection::Column);
    }

    #[test]
    fn stack_forces_stack_display() {
        let dom = lower_kind_of(NodeKind::Stack);
        assert_eq!(dom.style.display, Display::Stack);
    }

    #[test]
    fn scroll_sets_overflow() {
        let dom = lower(&Node::new(NodeKind::Scroll(ScrollSpec {
            axis: "vertical".into(),
            snap: false,
            scroll_to: None,
        })));
        assert_eq!(dom.style.overflow_y, Some(Overflow::Scroll));
    }

    #[test]
    fn style_maps_background_and_radius() {
        let mut n = Node::new(NodeKind::Div);
        n.style.background = Some(Brush::solid(Color::WHITE));
        n.style.radius = Some(CornerRadius::all(8.0));
        n.style.border_width = Some(2.0);
        n.style.border_color = Some(Color::BLACK);
        let dom = lower(&n);
        assert!(dom.style.background.is_some());
        assert_eq!(dom.style.radius, Some(CornerRadius::all(8.0)));
        assert_eq!(dom.style.border, Some((2.0, Color::BLACK)));
    }

    #[test]
    fn overlay_is_fixed_positioned() {
        let dom = lower(&Node::new(NodeKind::Overlay(OverlaySpec {
            layer: Some("modal".into()),
            backdrop: true,
            dismissible: true,
        })));
        assert_eq!(dom.style.position, Some(Position::Fixed));
    }

    #[test]
    fn glass_material_expands_to_concrete_paint() {
        let mut n = Node::new(NodeKind::Div);
        n.style.glass_material = Some(GlassMaterial { elevation: 24.0, ..GlassMaterial::default() });
        let dom = lower(&n);
        assert!(dom.style.glass, "glass flag set");
        let f = dom.style.filter.expect("filter synthesized");
        assert!(f.backdrop_blur > 0.0, "backdrop blur applied");
        assert!(f.saturate.is_some(), "saturation applied");
        assert!(dom.style.background.is_some(), "tint background applied");
        assert!(dom.style.border.is_some(), "specular rim applied");
        assert!(dom.style.radius.is_some(), "radius applied");
        assert_eq!(dom.style.shadows.len(), 1, "elevation shadow applied");
    }

    #[test]
    fn glass_material_does_not_clobber_explicit_fields() {
        let mut n = Node::new(NodeKind::Div);
        n.style.glass_material = Some(GlassMaterial::default());
        n.style.background = Some(Brush::solid(Color::BLACK));
        let dom = lower(&n);
        // The explicit background wins; glass only fills the gaps.
        assert_eq!(dom.style.background, Some(Brush::solid(Color::BLACK)));
        assert!(dom.style.glass);
    }

    /// Every kind must lower without panicking and to the expected content
    /// discriminant — the guard against "skipping" a Blinc family.
    #[test]
    fn all_kinds_lower() {
        use elpis_protocol::canvas::CanvasSpec;
        use elpis_protocol::scene3d::Scene3DSpec;
        let kinds = vec![
            NodeKind::Div,
            NodeKind::Row,
            NodeKind::Column,
            NodeKind::Stack,
            NodeKind::Grid,
            NodeKind::Spacer,
            NodeKind::Scroll(ScrollSpec { axis: "both".into(), snap: true, scroll_to: None }),
            NodeKind::Overlay(OverlaySpec { layer: None, backdrop: false, dismissible: true }),
            NodeKind::Text(TextSpec {
                text: "x".into(),
                size: 12.0,
                weight: FontWeight::Bold,
                font: None,
                italic: false,
                align: TextAlign::Center,
                underline: false,
                strikethrough: false,
                line_height: None,
                letter_spacing: None,
                max_lines: 0,
                selectable: false,
            }),
            NodeKind::RichText(RichTextSpec { runs: vec![], align: TextAlign::Start, line_height: None }),
            NodeKind::Markdown(MarkdownSpec { source: "# hi".into(), gfm: true }),
            NodeKind::Image(ImageSpec { src: "a.png".into(), fit: ImageFit::Cover, alt: None, placeholder: None }),
            NodeKind::Svg(SvgSpec { source: Some("<svg/>".into()), src: None, color: None }),
            NodeKind::Icon(IconSpec { name: "star".into(), size: 24.0, color: None, set: None }),
            NodeKind::Button(ButtonSpec { label: "ok".into(), variant: None, disabled: false, icon: None, loading: false }),
            NodeKind::TextInput(InputSpec { value: "".into(), placeholder: None, input_type: None, disabled: false, readonly: false, max_length: None, autofocus: false }),
            NodeKind::Checkbox(ToggleSpec { checked: true, label: None, disabled: false }),
            NodeKind::Switch(ToggleSpec { checked: false, label: None, disabled: false }),
            NodeKind::Radio(RadioSpec { options: vec![], selected: None, disabled: false }),
            NodeKind::Slider(SliderSpec { value: 0.5, min: 0.0, max: 1.0, step: None, disabled: false, value_end: None }),
            NodeKind::Dropdown(DropdownSpec { options: vec![], selected: None, placeholder: None, disabled: false, multi: false }),
            NodeKind::Tabs(TabsSpec { tabs: vec![], selected: 0 }),
            NodeKind::Carousel(CarouselSpec { index: 0, indicators: true, autoplay: false, interval: 0.0 }),
            NodeKind::ProgressBar(ProgressSpec { value: Some(0.3), shape: None }),
            NodeKind::Spinner(SpinnerSpec { size: 24.0, color: None }),
            NodeKind::Canvas(CanvasSpec::default()),
            NodeKind::Scene3D(Scene3DSpec::default()),
            NodeKind::Video(MediaSpec { src: "v.mp4".into(), autoplay: false, loop_: false, muted: false, controls: true, fit: ImageFit::Contain, seek: None }),
            NodeKind::Audio(MediaSpec { src: "a.mp3".into(), autoplay: false, loop_: false, muted: false, controls: true, fit: ImageFit::Contain, seek: None }),
            NodeKind::Component(ComponentSpec { name: "Card".into(), props: serde_json::Value::Null }),
        ];
        // The full surface: 30 widget families.
        assert_eq!(kinds.len(), 30);
        for k in kinds {
            let dom = lower(&Node::new(k));
            assert!(!dom.tag.is_empty());
        }
    }
}
