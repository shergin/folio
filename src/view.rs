//! `View`: the plainest view, a flexbox container.

use std::sync::Arc;

use hooke::{Children, LayoutState, Parts, PartsMut};

use crate::family::Family;
use crate::kind::{ShadowNodeKind, child_views};
use crate::props::{Props, ViewProps};
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};

/// A view that lays out its children with flexbox.
#[derive(Clone, Debug)]
pub struct ViewShadowNode {
    family: Arc<Family>,
    props: Arc<ViewProps>,
    children: Children<ShadowNodeKind>,
    layout: LayoutState,
}

impl ViewShadowNode {
    /// The first revision of a new view.
    pub fn new(props: ViewProps, children: impl IntoIterator<Item = Arc<ShadowNodeKind>>) -> Self {
        let family = Family::new();
        let children = child_views(&family, children);
        ViewShadowNode { family, props: Arc::new(props), children, layout: LayoutState::container() }
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
}

impl ShadowNode for ViewShadowNode {
    fn family(&self) -> &Arc<Family> {
        &self.family
    }

    fn props(&self) -> &dyn Props {
        &*self.props
    }
}

impl LayoutableShadowNode for ViewShadowNode {
    fn view_props(&self) -> &ViewProps {
        &self.props
    }

    fn layout_parts(&self) -> Parts<'_, ShadowNodeKind> {
        Parts { style: &self.props.layout, state: &self.layout, children: &self.children }
    }

    fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind> {
        PartsMut { style: &self.props.layout, state: &mut self.layout, children: &mut self.children }
    }
}
