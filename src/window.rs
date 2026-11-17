//! `Window`: a view that takes its space from the host.

use std::sync::{Arc, LazyLock};

use hooke::{Children, LayoutCx, LayoutState, Measure, Opt, Parts, PartsMut, Point, Position, RootConstraints};

use crate::family::Family;
use crate::kind::{ShadowNodeKind, child_views};
use crate::props::{LayoutStyle, Props, ViewProps};
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};

/// What the host says about one window: the space it offers and the facts of its surface.
#[derive(Clone, PartialEq, Debug)]
pub struct WindowState {
    /// The space the window offers each of its children.
    pub constraints: RootConstraints<f32>,
    /// Device pixels per layout unit.
    pub scale: f32,
    /// The user's text size preference, as a multiplier.
    pub font_scale: f32,
}

impl WindowState {
    /// A window of a fixed size, which its content fills.
    pub fn sized(width: f32, height: f32, scale: f32) -> WindowState {
        WindowState {
            constraints: RootConstraints::viewport(width, height).with_known(Opt::some(width), Opt::some(height)),
            scale,
            font_scale: 1.0,
        }
    }
}

/// A view that takes its space from the host instead of from its parent.
///
/// To its parent it is out of flow and has no size, wherever in the tree it stands. To its
/// children it offers the space the host reported, to each child on its own. A tree may
/// hold any number of windows, side by side or one inside the content of another.
///
/// The author declares a window by adding the node. The host opens it and reports its size
/// as the window's state; until then nothing inside it is laid out.
#[derive(Clone, Debug)]
pub struct WindowShadowNode {
    family: Arc<Family>,
    props: Arc<ViewProps>,
    state: Option<Arc<WindowState>>,
    children: Children<ShadowNodeKind>,
    layout: LayoutState,
}

/// The style every window shows its parent: taken out of the parent's flow.
static OUT_OF_FLOW: LazyLock<LayoutStyle> = LazyLock::new(|| LayoutStyle::default().with_position(Position::Absolute));

impl WindowShadowNode {
    /// The first revision of a new window, not yet opened by the host.
    pub fn new(props: ViewProps, children: impl IntoIterator<Item = Arc<ShadowNodeKind>>) -> Self {
        let family = Family::new();
        let children = child_views(&family, children);
        WindowShadowNode { family, props: Arc::new(props), state: None, children, layout: LayoutState::measured(0) }
    }

    /// The props, typed.
    pub fn props(&self) -> &ViewProps {
        &self.props
    }

    /// Replaces the props. The author's field.
    pub fn set_props(&mut self, props: ViewProps) {
        self.props = Arc::new(props);
    }

    /// Replaces the children. The author's field.
    pub fn set_children(&mut self, children: impl IntoIterator<Item = Arc<ShadowNodeKind>>) {
        self.children = child_views(&self.family, children);
    }

    /// What the host has reported, if it has opened the window.
    pub fn state(&self) -> Option<&WindowState> {
        self.state.as_deref()
    }

    /// Replaces the state. The host's field: the window was opened, resized, or moved to
    /// another screen. Sizes measured before no longer hold.
    pub fn set_state(&mut self, state: WindowState) {
        self.state = Some(Arc::new(state));
        self.layout.invalidate();
    }

    /// The window layout system: lays out each child, by flexbox, against the space the host
    /// reported. Does nothing until the host has opened the window.
    pub(crate) fn layout_children(
        &mut self,
        cx: &mut LayoutCx<'_, f32>,
        measure: &mut impl Measure<ShadowNodeKind>,
    ) {
        let Some(constraints) = self.state().map(|state| state.constraints) else { return };
        for slot in self.children.make_mut_untracked() {
            // A cancelled pass leaves the candidate to be dropped or laid out again.
            let _ = hooke::layout(&mut slot.node, constraints, cx, measure);
            slot.offset = Point::ZERO;
        }
    }
}

impl ShadowNode for WindowShadowNode {
    fn family(&self) -> &Arc<Family> {
        &self.family
    }

    fn props(&self) -> &dyn Props {
        &*self.props
    }
}

impl LayoutableShadowNode for WindowShadowNode {
    fn view_props(&self) -> &ViewProps {
        &self.props
    }

    fn layout_parts(&self) -> Parts<'_, ShadowNodeKind> {
        Parts { style: &OUT_OF_FLOW, state: &self.layout, children: &self.children }
    }

    fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind> {
        PartsMut { style: &OUT_OF_FLOW, state: &mut self.layout, children: &mut self.children }
    }
}
