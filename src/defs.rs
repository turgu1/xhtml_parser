//! xhtml_parser Definitions
//!
//! This file is part of the xhtml_parser project.
//! it contains definitions and types used throughout the parser.

use core::ops::Range;

pub type NodeIdx = u16;
pub type AttrIdx = u16;
pub type XmlIdx = usize;

pub type XmlRange = Range<XmlIdx>;
pub type NodeRange = Range<NodeIdx>;
pub type AttributeRange = Range<AttrIdx>;

#[derive(Debug)]
pub enum ParseXmlError {
    InvalidXml(String),
    NoMoreSpace,
    InternalError,
    NotEnoughMemory,
}
