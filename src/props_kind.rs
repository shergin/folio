//! The dispatch enum over props: the props of a node of any kind.

use std::any::Any;

use crate::image::ImageProps;
use crate::props::{Props, ViewProps};
use crate::text::TextProps;
use crate::text_span::TextSpanProps;

/// The props of a node of any kind.
///
/// This is to props what [`ShadowNodeKind`](crate::ShadowNodeKind) is to views. The props
/// types everyone uses are inline variants, and props of any other type go in
/// [`Other`](PropsKind::Other). The enum implements [`Props`] by forwarding to whichever
/// variant it holds.
///
/// A node holds its props as an `Arc<PropsKind>`. The enum is the props themselves; the
/// `Arc` around it is how the revisions of a node share them and how anything else keeps
/// them. [`ShadowNode::props_kind`](crate::ShadowNode::props_kind) hands that pointer out
/// for any node, so two nodes have the same props exactly when `Arc::ptr_eq` says so.
#[whichever::dispatch(Props)]
#[derive(Debug)]
pub enum PropsKind {
    /// The props of a plain view, and of a window.
    View(ViewProps),
    /// The props of a block of text.
    Text(TextProps),
    /// The props of a range inside text.
    TextSpan(TextSpanProps),
    /// The props of an image.
    Image(ImageProps),
    /// Props of a type this enum does not list.
    #[dispatch(deref)]
    Other(Box<dyn Props>),
}

impl PropsKind {
    /// The props as the concrete type `P`, if that is what is held. This is how a kind the
    /// enum does not list reads its own props back out of [`Other`](PropsKind::Other).
    pub fn downcast_ref<P: Props>(&self) -> Option<&P> {
        let props: &dyn Any = match self {
            PropsKind::View(props) => props,
            PropsKind::Text(props) => props,
            PropsKind::TextSpan(props) => props,
            PropsKind::Image(props) => props,
            PropsKind::Other(props) => &**props,
        };
        props.downcast_ref()
    }

    // What each basic kind reads its own props with. A node only ever holds the variant
    // its constructor and setter put there.

    pub(crate) fn view(&self) -> &ViewProps {
        match self {
            PropsKind::View(props) => props,
            _ => unreachable!("a view holds view props"),
        }
    }

    pub(crate) fn text(&self) -> &TextProps {
        match self {
            PropsKind::Text(props) => props,
            _ => unreachable!("a text holds text props"),
        }
    }

    pub(crate) fn text_span(&self) -> &TextSpanProps {
        match self {
            PropsKind::TextSpan(props) => props,
            _ => unreachable!("a span holds span props"),
        }
    }

    pub(crate) fn image(&self) -> &ImageProps {
        match self {
            PropsKind::Image(props) => props,
            _ => unreachable!("an image holds image props"),
        }
    }
}
