//! Folio: a new take on high-performance cross-platform UI frameworks.
//!
//! The crate is at its earliest stage. What exists is the shadow tree: the immutable,
//! structurally shared tree of nodes from which layout, mounting and everything else will
//! be computed. Nothing is drawn yet, and every definition here may still change.
//!
//! # The model
//!
//! - **A node is one immutable revision of one element.** [`ShadowNode`] is any node: an
//!   identity, its [`Family`], and what its author said, its props. Each kind of node has
//!   one props type, [`ShadowNode::Props`].
//! - **A view is a node that a layout system can size and place.** That is
//!   [`LayoutableShadowNode`]. Every view has [`ViewProps`] and layout state; nothing else
//!   has layout state.
//! - **One struct per kind, one enum over the views.** [`ShadowNodeKind`] holds a view of
//!   any kind. The basic kinds are inline variants, and any other kind fits in its last
//!   variant as a trait object. The tree is made of `Arc<ShadowNodeKind>`.
//! - **Layout systems meet at the view.** A kind says what it is to its parent and which
//!   layout system it runs for its children: [`ViewShadowNode`] runs flexbox,
//!   [`TextShadowNode`] runs text, [`WindowShadowNode`] hands out the space its host
//!   reported, [`ImageShadowNode`] has no children. A parent only ever asks a child view
//!   for a size and gives it an offset, so no system knows about another.
//! - **Ownership is mutation.** An owned node is mutable. A node behind an `Arc` is frozen,
//!   and `Arc::make_mut` copies it first when it is shared. Every `Clone` is shallow.
//! - **Every field has one writer.** The author writes props and children, the host writes
//!   state, the layout systems write layout.
//! - **A tree is a sequence of revisions.** [`ShadowTree`] holds the current [`Revision`];
//!   a candidate is built and laid out on any thread, then offered to
//!   [`ShadowTree::try_commit`]. [`edit`] makes the next revision of one view by identity.

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod family;
mod image;
mod kind;
mod layout;
mod props;
mod shadow_node;
mod text;
mod text_span;
mod text_style;
mod tree;
mod view;
mod window;

pub use family::{Family, FamilyId};
pub use image::{ImageProps, ImageShadowNode, ImageState};
pub use kind::ShadowNodeKind;
pub use layout::LayoutSystems;
pub use props::{Accessibility, Appearance, Color, LayoutStyle, Props, ViewProps};
pub use shadow_node::{CloneBoxed, LayoutableShadowNode, ShadowNode};
pub use text::{Inline, TextItem, TextProps, TextShadowNode};
pub use text_span::{TextSpanProps, TextSpanShadowNode};
pub use text_style::{ParagraphStyle, TextAttributes, TextStyle};
pub use tree::{Revision, ShadowTree, Stale, edit};
pub use view::ViewShadowNode;
pub use window::{WindowShadowNode, WindowState};
