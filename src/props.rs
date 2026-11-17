//! Props: what an author says about a node.
//!
//! Props are plain data. A props type is a record of groups plus fields of its own. A group
//! is a struct of props that one consumer reads and that tends to change together; each
//! group sits behind its own `Arc`, so changing one leaves the others pointer-equal. Where
//! Fabric derives one props class from another, a props type here contains the other.

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// The complete, immutable description of one node, as its author wrote it.
///
/// A marker for the type-erased view that [`ShadowNode::props`](crate::ShadowNode::props)
/// gives. Typed code reads the props of a concrete kind instead.
pub trait Props: Any + Debug + Send + Sync {}

/// Everything flexbox reads. The layout engine's style, unchanged.
pub type LayoutStyle = hooke::Style<f32>;

/// A colour as RGBA, eight bits per channel.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Color(pub u32);

/// Everything the painter reads.
#[derive(Clone, PartialEq, Debug)]
pub struct Appearance {
    /// How opaque the view and its content are, from 0 to 1.
    pub opacity: f32,
    /// The fill behind the view's content.
    pub background: Color,
    /// The radius of all four corners.
    pub corner_radius: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance { opacity: 1.0, background: Color::default(), corner_radius: 0.0 }
    }
}

/// Everything assistive technology is told.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Accessibility {
    /// What the view is announced as.
    pub label: Option<Arc<str>>,
    /// Whether the view and its content are kept from assistive technology.
    pub hidden: bool,
}

/// The props every view has: three groups, three pointers.
#[derive(Clone, Debug, Default)]
pub struct ViewProps {
    /// How the view is sized and placed, and how it places its children.
    pub layout: Arc<LayoutStyle>,
    /// How the view is painted.
    pub appearance: Arc<Appearance>,
    /// How the view is presented to assistive technology.
    pub accessibility: Arc<Accessibility>,
}

impl Props for ViewProps {}
