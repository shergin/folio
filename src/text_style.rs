//! Text styling: what an author may say, and what the text system receives.

use std::sync::Arc;

use crate::props::Color;

/// What an author may say about text at any level of a text subtree.
///
/// Every field may be left unset, and an unset field inherits from the enclosing level.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct TextStyle {
    /// The colour of the glyphs.
    pub color: Option<Color>,
    /// The name of the font family.
    pub font_family: Option<Arc<str>>,
    /// The font size, in layout units.
    pub font_size: Option<f32>,
    /// The font weight, from 1 to 1000.
    pub font_weight: Option<u16>,
    /// Whether the text is set in italics.
    pub italic: Option<bool>,
    /// Extra space after each character, in layout units.
    pub letter_spacing: Option<f32>,
}

/// What the text system receives for one run of characters: every field decided.
#[derive(Clone, PartialEq, Debug)]
pub struct TextAttributes {
    /// The colour of the glyphs.
    pub color: Color,
    /// The name of the font family.
    pub font_family: Arc<str>,
    /// The font size, in layout units.
    pub font_size: f32,
    /// The font weight, from 1 to 1000.
    pub font_weight: u16,
    /// Whether the text is set in italics.
    pub italic: bool,
    /// Extra space after each character, in layout units.
    pub letter_spacing: f32,
}

impl TextStyle {
    /// One step of the cascade: this style laid over what it inherits.
    pub fn resolve(&self, inherited: &TextAttributes) -> TextAttributes {
        TextAttributes {
            color: self.color.unwrap_or(inherited.color),
            font_family: self.font_family.clone().unwrap_or_else(|| inherited.font_family.clone()),
            font_size: self.font_size.unwrap_or(inherited.font_size),
            font_weight: self.font_weight.unwrap_or(inherited.font_weight),
            italic: self.italic.unwrap_or(inherited.italic),
            letter_spacing: self.letter_spacing.unwrap_or(inherited.letter_spacing),
        }
    }
}

/// Settings of a text block as a whole. They do not cascade into spans.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ParagraphStyle {
    /// The most lines the block may take; the rest is cut off.
    pub max_lines: Option<u32>,
}
