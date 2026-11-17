//! The dispatch enum: a view of any kind.

use std::sync::Arc;

use hooke::{Children, LayoutNode, Parts, PartsMut};

use crate::family::Family;
use crate::image::ImageShadowNode;
use crate::props::{Props, ViewProps};
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};
use crate::text::TextShadowNode;
use crate::view::ViewShadowNode;
use crate::window::WindowShadowNode;

/// A view of any kind. This is the node type the tree is made of, and the node type the
/// flexbox engine lays out.
///
/// The kinds everyone uses are inline: one allocation per node, and a `match` per call that
/// the optimiser sees through. Any other kind implements [`ShadowNode`] and
/// [`LayoutableShadowNode`] and goes in [`Other`](ShadowNodeKind::Other), at the price of
/// one more pointer and a virtual call.
///
/// Typed access is a `match`. Code that does not care about the kind calls the two traits,
/// which the enum implements by forwarding to its variant.
#[whichever::dispatch(ShadowNode)]
#[whichever::dispatch(LayoutableShadowNode)]
#[derive(Clone, Debug)]
pub enum ShadowNodeKind {
    /// A view that takes its space from the host.
    Window(WindowShadowNode),
    /// A view that lays out its children with flexbox.
    View(ViewShadowNode),
    /// A view that lays out its content as text.
    Text(TextShadowNode),
    /// A view that shows a picture.
    Image(ImageShadowNode),
    /// A view of a kind this enum does not list.
    #[dispatch(deref)]
    Other(Box<dyn LayoutableShadowNode>),
}

// A view is 248 bytes, 192 of them layout state.
const _: () = assert!(size_of::<ShadowNodeKind>() == 248);

// A revision can be read from any thread without a lock.
const _: fn() = || {
    fn shareable<T: Send + Sync>() {}
    shareable::<ShadowNodeKind>();
};

/// The whole of the flexbox embedding: the engine reads and writes a view through the three
/// parts it hands out.
impl LayoutNode for ShadowNodeKind {
    type Scalar = f32;

    fn parts(&self) -> Parts<'_, Self> {
        self.layout_parts()
    }

    fn parts_mut(&mut self) -> PartsMut<'_, Self> {
        self.layout_parts_mut()
    }
}

/// A child list of `nodes` for a parent of the given family, each child placed under it.
pub(crate) fn child_views(
    parent: &Arc<Family>,
    nodes: impl IntoIterator<Item = Arc<ShadowNodeKind>>,
) -> Children<ShadowNodeKind> {
    let children = Children::from_nodes(nodes);
    for slot in children.slots() {
        slot.node.family().place_under(parent);
    }
    children
}
