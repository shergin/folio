//! `Text`: a view that lays out its content as text.

use std::sync::Arc;

use hooke::{Children, LayoutState, Parts, PartsMut};

use crate::family::Family;
use crate::kind::{ShadowNodeKind, child_views};
use crate::props::{Props, ViewProps};
use crate::shadow_node::{LayoutableShadowNode, ShadowNode};
use crate::text_span::TextSpanShadowNode;
use crate::text_style::{ParagraphStyle, TextAttributes, TextStyle};

/// Props of a block of text: a view, and the root of a text cascade.
#[derive(Clone, Debug, Default)]
pub struct TextProps {
    /// The props every view has.
    pub view: ViewProps,
    /// What the block says about all of its text.
    pub style: Arc<TextStyle>,
    /// Settings of the block as a whole.
    pub paragraph: Arc<ParagraphStyle>,
}

impl Props for TextProps {}

/// One child in a text context. `[Inline]` is the child list of the text system, as
/// `hooke::Children` is the child list of flexbox.
#[derive(Clone, Debug)]
pub enum Inline {
    /// Characters, styled by whatever encloses them. Not a node: a run has no identity.
    Run(Arc<str>),
    /// A range that restyles what it contains.
    Span(Arc<TextSpanShadowNode>),
    /// A view placed in the text: the one at this position among the attachments of the
    /// enclosing [`TextShadowNode`].
    Attachment(usize),
}

/// Places every span of `content` under the family of the node that holds it.
pub(crate) fn place_spans(parent: &Arc<Family>, content: &[Inline]) {
    for inline in content {
        if let Inline::Span(span) = inline {
            span.family().place_under(parent);
        }
    }
}

/// What the text system is handed: the content with the cascade applied.
#[derive(Clone, PartialEq, Debug)]
pub enum TextItem {
    /// Characters, with every attribute decided.
    Characters(Arc<str>, TextAttributes),
    /// The attachment at this position.
    Attachment(usize),
}

/// A view that lays out its content as text. To its parent it is a measured leaf.
///
/// Its children are inline: runs of characters, spans, and views placed in the text. The
/// views are kept apart in `attachments`, because they are views like any other: the text
/// system sizes them and gives them offsets, and everything that walks views finds them
/// there. The inline content marks where each one goes.
#[derive(Clone, Debug)]
pub struct TextShadowNode {
    family: Arc<Family>,
    props: Arc<TextProps>,
    content: Arc<[Inline]>,
    attachments: Children<ShadowNodeKind>,
    layout: LayoutState,
}

impl TextShadowNode {
    /// The first revision of a new block of text.
    pub fn new(
        props: TextProps,
        content: impl IntoIterator<Item = Inline>,
        attachments: impl IntoIterator<Item = Arc<ShadowNodeKind>>,
    ) -> Self {
        let family = Family::new();
        let content: Arc<[Inline]> = content.into_iter().collect();
        place_spans(&family, &content);
        let attachments = child_views(&family, attachments);
        TextShadowNode { family, props: Arc::new(props), content, attachments, layout: LayoutState::measured(0) }
    }

    /// Replaces the props. The author's field. Sizes measured before no longer hold.
    pub fn set_props(&mut self, props: TextProps) {
        self.props = Arc::new(props);
        self.layout.invalidate();
    }

    /// The inline children, in order.
    pub fn content(&self) -> &[Inline] {
        &self.content
    }

    /// Replaces the inline children and the views among them. The author's field. Sizes
    /// measured before no longer hold.
    pub fn set_content(
        &mut self,
        content: impl IntoIterator<Item = Inline>,
        attachments: impl IntoIterator<Item = Arc<ShadowNodeKind>>,
    ) {
        self.content = content.into_iter().collect();
        place_spans(&self.family, &self.content);
        self.attachments = child_views(&self.family, attachments);
        self.layout.invalidate();
    }

    /// Flattens the content into what the text system is handed: characters with every
    /// attribute decided, and attachments by position.
    ///
    /// `inherited` is what applies outside the block, such as the platform's default font.
    pub fn items(&self, inherited: &TextAttributes) -> Vec<TextItem> {
        fn walk(content: &[Inline], attributes: &TextAttributes, items: &mut Vec<TextItem>) {
            for inline in content {
                match inline {
                    Inline::Run(characters) => items.push(TextItem::Characters(characters.clone(), attributes.clone())),
                    Inline::Span(span) => walk(span.children(), &span.props().style.resolve(attributes), items),
                    Inline::Attachment(index) => items.push(TextItem::Attachment(*index)),
                }
            }
        }
        let mut items = Vec::new();
        walk(&self.content, &self.props.style.resolve(inherited), &mut items);
        items
    }
}

impl ShadowNode for TextShadowNode {
    type Props = TextProps;

    fn family(&self) -> &Arc<Family> {
        &self.family
    }

    fn props(&self) -> &TextProps {
        &self.props
    }
}

impl LayoutableShadowNode for TextShadowNode {
    fn view_props(&self) -> &ViewProps {
        &self.props.view
    }

    fn layout_parts(&self) -> Parts<'_, ShadowNodeKind> {
        Parts { style: &self.props.view.layout, state: &self.layout, children: &self.attachments }
    }

    fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind> {
        PartsMut { style: &self.props.view.layout, state: &mut self.layout, children: &mut self.attachments }
    }
}
