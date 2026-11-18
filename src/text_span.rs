//! `TextSpan`: a range inside text that restyles what it contains.

use std::sync::Arc;

use crate::family::Family;
use crate::props::{Props, ViewProps};
use crate::props_kind::PropsKind;
use crate::shadow_node::ShadowNode;
use crate::text::{Inline, place_spans};
use crate::text_style::TextStyle;

/// Props of a range inside text: a step of the text cascade and nothing else.
#[derive(Clone, Debug, Default)]
pub struct TextSpanProps {
    /// What this range says about its text.
    pub style: Arc<TextStyle>,
}

impl Props for TextSpanProps {
    fn view_props(&self) -> Option<&ViewProps> {
        None
    }
}

/// A range inside text that restyles what it contains.
///
/// A node but not a view: it has an identity, a style and inline children, and no layout
/// state, because nothing ever asks a span for a size. It is 32 bytes where a view is 248,
/// and it lives in the inline child list of its text or of an enclosing span.
#[derive(Clone, Debug)]
pub struct TextSpanShadowNode {
    family: Family,
    props: Arc<TextSpanProps>,
    children: Arc<[Inline]>,
}

const _: () = assert!(size_of::<TextSpanShadowNode>() == 32);

impl TextSpanShadowNode {
    /// The first revision of a new span.
    pub fn new(props: TextSpanProps, children: impl IntoIterator<Item = Inline>) -> Self {
        let family = Family::new();
        let children: Arc<[Inline]> = children.into_iter().collect();
        place_spans(&family, &children);
        TextSpanShadowNode { family, props: Arc::new(props), children }
    }

    /// What the span contains, in order.
    pub fn children(&self) -> &[Inline] {
        &self.children
    }
}

impl ShadowNode for TextSpanShadowNode {
    type Props = TextSpanProps;

    fn family(&self) -> &Family {
        &self.family
    }

    fn props(&self) -> &TextSpanProps {
        &self.props
    }

    fn props_kind(&self) -> PropsKind {
        PropsKind::TextSpan(self.props.clone())
    }
}
