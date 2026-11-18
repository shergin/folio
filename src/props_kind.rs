//! The dispatch enum over props: the props of a node of any kind.

use std::sync::Arc;

use crate::image::ImageProps;
use crate::props::{Props, ViewProps};
use crate::text::TextProps;
use crate::text_span::TextSpanProps;

/// The props of a node of any kind.
///
/// [`ShadowNode::props`](crate::ShadowNode::props) is typed, so it needs the kind of the
/// node to be known. This is what [`ShadowNode::props_kind`](crate::ShadowNode::props_kind)
/// hands out when it is not: the same props, in a form that can be matched on.
///
/// The props are shared with the node, not copied. A `PropsKind` can therefore be kept
/// after the node is gone, and [`ptr_eq`](PropsKind::ptr_eq) tells in one comparison
/// whether two nodes have the same props.
///
/// The props types everyone uses are variants; props of any other type go in
/// [`Other`](PropsKind::Other). The enum implements [`Props`] by forwarding to whichever
/// variant it holds.
#[whichever::dispatch(Props)]
#[derive(Clone, Debug)]
pub enum PropsKind {
    /// The props of a plain view, and of a window.
    #[dispatch(deref)]
    View(Arc<ViewProps>),
    /// The props of a block of text.
    #[dispatch(deref)]
    Text(Arc<TextProps>),
    /// The props of a range inside text.
    #[dispatch(deref)]
    TextSpan(Arc<TextSpanProps>),
    /// The props of an image.
    #[dispatch(deref)]
    Image(Arc<ImageProps>),
    /// Props of a type this enum does not list.
    #[dispatch(deref)]
    Other(Arc<dyn Props>),
}

impl PropsKind {
    /// True if both hold the very same props, which is what two revisions of a node hold
    /// when the author did not change them.
    pub fn ptr_eq(&self, other: &PropsKind) -> bool {
        match (self, other) {
            (PropsKind::View(a), PropsKind::View(b)) => Arc::ptr_eq(a, b),
            (PropsKind::Text(a), PropsKind::Text(b)) => Arc::ptr_eq(a, b),
            (PropsKind::TextSpan(a), PropsKind::TextSpan(b)) => Arc::ptr_eq(a, b),
            (PropsKind::Image(a), PropsKind::Image(b)) => Arc::ptr_eq(a, b),
            (PropsKind::Other(a), PropsKind::Other(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
