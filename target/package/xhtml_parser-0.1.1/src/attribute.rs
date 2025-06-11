//! Attribute handling for XML/XHTML parsing.
//!
//! This module provides structures and functionality for working with XML attributes,
//! including individual attribute access and iteration over collections of attributes.

use std::fmt::{self, Debug};

use crate::defs::XmlRange;
use crate::document::Document;
use crate::node::Node;
use crate::node_type::NodeType;

/// Information about an XML attribute, storing name and value ranges within the document.
///
/// This struct holds references to positions in the source document where the attribute
/// name and value are located, allowing for efficient string retrieval without copying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeInfo {
    name: XmlRange,
    value: XmlRange,
}

impl AttributeInfo {
    /// Creates a new AttributeInfo with the specified name and value ranges.
    ///
    /// # Arguments
    /// * `name` - The range in the document containing the attribute name
    /// * `value` - The range in the document containing the attribute value
    ///
    /// # Returns
    /// A new AttributeInfo instance
    pub fn new(name: XmlRange, value: XmlRange) -> Self {
        AttributeInfo { name, value }
    }
}

/// A reference to an XML attribute within a document.
///
/// This struct provides access to an attribute's name and value by combining
/// a reference to the document with attribute information.
pub struct Attribute<'a> {
    doc: &'a Document,
    data: &'a AttributeInfo,
}

impl<'xml> Attribute<'xml> {
    /// Returns the name of the attribute as a string slice.
    ///
    /// # Returns
    /// A string slice containing the attribute name
    #[inline]
    pub fn name(&self) -> &str {
        self.doc.get_str_from_range(&self.data.name)
    }

    /// Returns true if the attribute's name matches the given string.
    #[inline]
    pub fn is(&self, name: &str) -> bool {
        self.name() == name
    }

    /// Returns the value of the attribute as a string slice.
    ///
    /// # Returns  
    /// A string slice containing the attribute value
    #[inline]
    pub fn value(&self) -> &'xml str {
        self.doc.get_str_from_range(&self.data.value)
    }
}

/// Custom Debug implementation for Attribute that displays name and value.
impl fmt::Debug for Attribute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Attribute {{ name: \"{}\", value: \"{}\" }}",
            self.name(),
            self.value()
        )
    }
}

/// An iterator over the attributes of an XML node.
///
/// This iterator provides access to all attributes belonging to a specific node,
/// yielding `Attribute` instances for each attribute found.
#[derive(Clone)]
pub struct Attributes<'a> {
    doc: &'a Document,
    attrs: core::slice::Iter<'a, AttributeInfo>,
}

impl<'a> Attributes<'a> {
    /// Creates a new Attributes iterator for the given node.
    ///
    /// # Arguments
    /// * `node` - The node whose attributes should be iterated over
    ///
    /// # Returns
    /// An Attributes iterator that will yield all attributes of the node
    #[inline]
    pub fn new(node: &Node<'a>) -> Attributes<'a> {
        let attrs = match node.node_info.node_type() {
            NodeType::Element { ref attributes, .. } => {
                &node.doc.attributes[attributes.start as usize..attributes.end as usize]
            }
            _ => &[],
        };

        Attributes {
            doc: node.doc,
            attrs: attrs.iter(),
        }
    }
}

/// Iterator implementation that yields Attribute instances.
impl<'a> Iterator for Attributes<'a> {
    type Item = Attribute<'a>;

    /// Returns the next attribute in the iteration.
    ///
    /// # Returns
    /// Some(Attribute) if there are more attributes, None otherwise
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.attrs.next().map(|attr| Attribute {
            doc: self.doc,
            data: attr,
        })
    }

    /// Returns the nth attribute, skipping n-1 attributes.
    ///
    /// # Arguments
    /// * `n` - The number of attributes to skip
    ///
    /// # Returns
    /// Some(Attribute) if the nth attribute exists, None otherwise
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.attrs.nth(n).map(|attr| Attribute {
            doc: self.doc,
            data: attr,
        })
    }

    /// Returns bounds on the remaining length of the iterator.
    ///
    /// # Returns
    /// A tuple of (lower_bound, upper_bound) where upper_bound is Some(exact_size)
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.attrs.size_hint()
    }
}

/// DoubleEndedIterator implementation allowing reverse iteration.
impl<'a> DoubleEndedIterator for Attributes<'a> {
    /// Returns the next attribute from the end of the iteration.
    ///
    /// # Returns
    /// Some(Attribute) if there are more attributes from the back, None otherwise
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.attrs.next_back().map(|attr| Attribute {
            doc: self.doc,
            data: attr,
        })
    }
}

/// ExactSizeIterator implementation indicating the iterator knows its exact length.
impl<'a> ExactSizeIterator for Attributes<'a> {
    // #[inline]
    // fn len(&self) -> usize {
    //     self.attrs.len()
    // }
}
