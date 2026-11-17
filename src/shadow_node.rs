//! The two traits: any node, and a node that can be laid out.
//!
//! Both are dispatchable, so [`ShadowNodeKind`] implements them by forwarding to whichever
//! variant it holds. Both are dyn compatible, so a kind nobody listed fits in the enum's
//! last variant.

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use hooke::{Parts, PartsMut};

use crate::family::Family;
use crate::kind::ShadowNodeKind;
use crate::props::{Props, ViewProps};

/// Any node: an identity and what its author said.
#[whichever::dispatchable]
pub trait ShadowNode: Any + Debug + Send + Sync {
    /// Who this node is, across revisions.
    fn family(&self) -> &Arc<Family>;

    /// What the author said, with its type erased. Typed code reads the props of a concrete
    /// kind instead; this is for code that only compares or prints them.
    fn props(&self) -> &dyn Props;
}

/// A view: a node that a layout system can size and place.
///
/// This is the whole interface between layout systems. Whatever system a parent runs, it
/// asks a child view for a size under constraints and then gives it an offset. Whatever
/// system the child runs inside is the child's business.
#[whichever::dispatchable]
pub trait LayoutableShadowNode: ShadowNode + CloneBoxed {
    /// The props every view has.
    fn view_props(&self) -> &ViewProps;

    /// What a layout pass reads: the style the view shows its parent, its cached sizes, and
    /// its child views with their offsets.
    fn layout_parts(&self) -> Parts<'_, ShadowNodeKind>;

    /// The same, for a pass that writes. Reached only through copy-on-write.
    fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind>;
}

/// Makes a boxed view clonable, which copy-on-write needs of [`ShadowNodeKind`].
///
/// Implemented for every view that is `Clone`. Nobody writes it by hand.
pub trait CloneBoxed {
    /// A shallow copy of this view, boxed.
    fn clone_boxed(&self) -> Box<dyn LayoutableShadowNode>;
}

impl<T: LayoutableShadowNode + Clone> CloneBoxed for T {
    fn clone_boxed(&self) -> Box<dyn LayoutableShadowNode> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn LayoutableShadowNode> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}
