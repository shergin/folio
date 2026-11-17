//! Where flexbox hands over to the other layout systems.

use hooke::{LayoutCx, Measure, MeasureInput, Measured};

use crate::kind::ShadowNodeKind;

/// What runs when flexbox reaches a view it does not lay out itself.
///
/// Each arm is another layout system. Pass it to `hooke::layout` as the measurer, with the
/// root of a candidate. Later it will hold what the systems need from outside, such as the
/// text shaper.
#[derive(Debug, Default)]
pub struct LayoutSystems;

impl Measure<ShadowNodeKind> for LayoutSystems {
    fn measure(
        &mut self,
        node: &mut ShadowNodeKind,
        _input: MeasureInput<f32>,
        cx: &mut LayoutCx<'_, f32>,
    ) -> Measured<f32> {
        match node {
            // The window system. The window itself takes no space where it stands.
            ShadowNodeKind::Window(window) => {
                window.layout_children(cx, self);
                Measured::size(0.0, 0.0)
            }
            // The text system is not written yet. It will shape `text.items(..)` and lay out
            // the attachments the way a window lays out its children.
            ShadowNodeKind::Text(_) => Measured::size(0.0, 0.0),
            ShadowNodeKind::Image(image) => {
                let size = image.natural_size();
                Measured::size(size.width, size.height)
            }
            // A flexbox container is never measured, and other kinds have no measurer yet.
            ShadowNodeKind::View(_) | ShadowNodeKind::Other(_) => Measured::default(),
        }
    }
}
