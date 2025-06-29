//! xhtml_parser Definitions
//!
//! This file is part of the xhtml_parser project.
//! it contains definitions and types used throughout the parser.

use core::ops::Range;

#[cfg(all(feature = "small_node_count", feature = "medium_node_count"))]
compile_error!("feature \"small_node_count\" and feature \"medium_node_count\" cannot be enabled at the same time");

#[cfg(all(feature = "small_node_count", feature = "large_node_count"))]
compile_error!("feature \"small_node_count\" and feature \"large_node_count\" cannot be enabled at the same time");

#[cfg(all(feature = "medium_node_count", feature = "large_node_count"))]
compile_error!("feature \"medium_node_count\" and feature \"large_node_count\" cannot be enabled at the same time");

#[cfg(not(any(
    feature = "small_node_count",
    feature = "medium_node_count",
    feature = "large_node_count"
)))]
compile_error!("one of the features \"small_node_count\", \"medium_node_count\", or \"large_node_count\" must be enabled");

#[cfg(feature = "small_node_count")]
pub type NodeIdx = u16;

#[cfg(feature = "medium_node_count")]
pub type NodeIdx = u32;

#[cfg(feature = "large_node_count")]
pub type NodeIdx = u64;

pub type AttrIdx = u16;
pub type XmlIdx = u32;

#[cfg(feature = "use_cstr")]
pub type XmlLocation = XmlIdx;

#[cfg(not(feature = "use_cstr"))]
pub type XmlLocation = Range<XmlIdx>;

pub type NodeRange = Range<NodeIdx>;
pub type AttributeRange = Range<AttrIdx>;

#[derive(Debug)]
pub enum ParseXmlError {
    InvalidXml(String),
    NoMoreSpace,
    InternalError,
    NotEnoughMemory,
}

impl std::fmt::Display for ParseXmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseXmlError::InvalidXml(msg) => write!(f, "Invalid XML: {}", msg),
            ParseXmlError::NoMoreSpace => write!(f, "No more space available for parsing"),
            ParseXmlError::InternalError => write!(f, "Internal error occurred during parsing"),
            ParseXmlError::NotEnoughMemory => {
                write!(f, "Not enough memory to complete the operation")
            }
        }
    }
}
