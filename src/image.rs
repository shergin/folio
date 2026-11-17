//! `Image`: a view that shows a picture.

use std::sync::Arc;

use hooke::{Children, LayoutState, Parts, PartsMut, Size};

use crate::family::Family;
use crate::kind::ShadowNodeKind;
use crate::props::{Color, Props, ViewProps};
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};

/// Props of an image: a view, and what to show in it.
#[derive(Clone, Debug, Default)]
pub struct ImageProps {
    /// The props every view has.
    pub view: ViewProps,
    /// Where the picture comes from.
    pub source: Option<Arc<str>>,
    /// A colour that replaces every opaque pixel.
    pub tint: Option<Color>,
}

impl Props for ImageProps {}

/// What the host learned about an image after the author described it.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ImageState {
    /// The size of the picture itself, once it is known.
    pub natural_size: Option<Size<f32>>,
}

/// A view with nothing inside it. To its parent it is a measured leaf, sized by the picture.
#[derive(Clone, Debug)]
pub struct ImageShadowNode {
    family: Family,
    props: Arc<ImageProps>,
    state: Option<Arc<ImageState>>,
    /// Always empty. A layout pass borrows a child list from every view.
    children: Children<ShadowNodeKind>,
    layout: LayoutState,
}

impl ImageShadowNode {
    /// The first revision of a new image.
    pub fn new(props: ImageProps) -> Self {
        ImageShadowNode {
            family: Family::new(),
            props: Arc::new(props),
            state: None,
            children: Children::empty(),
            layout: LayoutState::measured(0),
        }
    }

    /// Replaces the props. The author's field.
    pub fn set_props(&mut self, props: ImageProps) {
        self.props = Arc::new(props);
        self.layout.invalidate();
    }

    /// What the host has reported, if anything yet.
    pub fn state(&self) -> Option<&ImageState> {
        self.state.as_deref()
    }

    /// Replaces the state. The host's field. Sizes measured before no longer hold.
    pub fn set_state(&mut self, state: ImageState) {
        self.state = Some(Arc::new(state));
        self.layout.invalidate();
    }

    /// The size the picture asks for: its own, or nothing while that is unknown.
    pub(crate) fn natural_size(&self) -> Size<f32> {
        self.state().and_then(|state| state.natural_size).unwrap_or_default()
    }
}

impl ShadowNode for ImageShadowNode {
    type Props = ImageProps;

    fn family(&self) -> &Family {
        &self.family
    }

    fn props(&self) -> &ImageProps {
        &self.props
    }
}

impl LayoutableShadowNode for ImageShadowNode {
    fn view_props(&self) -> &ViewProps {
        &self.props.view
    }

    fn layout_parts(&self) -> Parts<'_, ShadowNodeKind> {
        Parts { style: &self.props.view.layout, state: &self.layout, children: &self.children }
    }

    fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind> {
        PartsMut { style: &self.props.view.layout, state: &mut self.layout, children: &mut self.children }
    }
}
