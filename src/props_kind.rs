//! The dispatch enum over props: props of any kind.

use std::any::Any;

use crate::image::ImageProps;
use crate::props::{Props, ViewProps};
use crate::text::TextProps;
use crate::text_span::TextSpanProps;

/// Props of any kind, as one value.
///
/// This is to props what [`ShadowNodeKind`](crate::ShadowNodeKind) is to views. The props
/// types everyone uses are inline variants, and props of any other type go in
/// [`Other`](PropsKind::Other). The enum implements [`Props`] by forwarding to whichever
/// variant it holds, and converts from each of its payloads.
///
/// Nodes do not hold their props in this form: each kind holds its own props type. The
/// enum is for code that has to carry props without knowing which kind they belong to.
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
    /// The props as the concrete type `P`, if that is what is held. This is how props the
    /// enum does not list are read back out of [`Other`](PropsKind::Other).
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
}
