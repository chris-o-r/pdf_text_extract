use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPageText {
    pub page_index: usize,
    pub text: String,
    pub bounding_box: BoundingBox,
}

impl PdfPageText {
    pub fn new(page_index: usize, text: String, bounding_box: BoundingBox) -> PdfPageText {
        PdfPageText {
            page_index,
            text,
            bounding_box,
        }
    }

    pub fn combine(&self, other: &PdfPageText) -> PdfPageText {
        PdfPageText {
            page_index: self.page_index,
            text: format!("{} {}", self.text, other.text),
            bounding_box: self.bounding_box.combine(&other.bounding_box),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl BoundingBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> BoundingBox {
        BoundingBox {
            x,
            y,
            width,
            height,
        }
    }

    pub fn combine(&self, other: &BoundingBox) -> BoundingBox {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let width = (self.x + self.width).max(other.x + other.width) - x;
        let height = (self.y + self.height).max(other.y + other.height) - y;

        BoundingBox {
            x,
            y,
            width,
            height,
        }
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BoundingBox({}, {}, {}, {})",
            self.x, self.y, self.width, self.height
        )
    }
}

impl fmt::Display for PdfPageText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Page {}: '{}' at {}",
            self.page_index, self.text, self.bounding_box
        )
    }
}

#[derive(Debug, Clone)]
pub enum ChunkingStrategy {
    Paragraph,
    Line,
}
impl clap::ValueEnum for ChunkingStrategy {
    fn value_variants<'a>() -> &'a [Self] {
        &[ChunkingStrategy::Paragraph, ChunkingStrategy::Line]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            ChunkingStrategy::Paragraph => Some(clap::builder::PossibleValue::new("paragraph")),
            ChunkingStrategy::Line => Some(clap::builder::PossibleValue::new("line")),
        }
    }
}

impl fmt::Display for ChunkingStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChunkingStrategy::Paragraph => write!(f, "Paragraph"),
            ChunkingStrategy::Line => write!(f, "Line"),
        }
    }
}

impl PartialEq for ChunkingStrategy {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ChunkingStrategy::Paragraph, ChunkingStrategy::Paragraph) => true,
            (ChunkingStrategy::Line, ChunkingStrategy::Line) => true,
            _ => false,
        }
    }
}
