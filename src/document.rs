//! Document module for the `xhtml-parser` crate.
//!
//!

#![allow(unused, dead_code)]

use log::{debug, warn};

use core::ops::Range;
use memchr::memchr_iter;
use std::fmt::{self, Debug};

use crate::attribute::{Attribute, AttributeInfo, Attributes};
use crate::defs::{AttrIdx, AttributeRange, NodeIdx, NodeRange, ParseXmlError, XmlIdx, XmlRange};
use crate::node::Node;
use crate::node_info::NodeInfo;
use crate::node_type::NodeType;
use crate::parser;

/// Represents a parsed XML document.
///
/// The `Document` struct contains a vector of `NodeInfo` representing the nodes in the document,
/// a vector of `AttributeInfo` representing the attributes, and the raw XML content as a byte vector.
/// It provides methods to create a new document from XML content, retrieve the root node,
/// get nodes by index, add new nodes and attributes, and access the XML content.

#[derive(PartialEq, Eq)]
pub struct Document {
    pub nodes: Vec<NodeInfo>,
    pub attributes: Vec<AttributeInfo>,
    pub xml: Vec<u8>,
}

impl Document {
    /// Creates a new `Document` from the provided XML content.
    ///
    /// # Arguments
    /// - `xml`: A byte vector containing the XML content to be parsed. the Document instance becomes the owner of the XML content
    ///
    /// # Returns
    /// - `Ok(Document)`: If the XML content is successfully parsed and a document is created.
    /// - `Err(ParseXmlError)`: If there is an error during parsing, such as invalid XML or insufficient memory.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root = document.root().unwrap();
    /// assert_eq!(root.tag_name(), "root");
    /// let child = root.first_child().unwrap();
    /// assert_eq!(child.tag_name(), "child");
    /// let child_text = child.first_child().unwrap();
    /// assert_eq!(child_text.text().unwrap(), "Text");
    /// ```
    /// # Notes
    /// - The `Document` struct is designed to handle XML documents and provides methods for navigating the document tree.
    /// - The `new` method estimates the number of nodes and attributes based on the XML content and allocates memory accordingly.
    ///   This is done to optimize performance and reduce memory reallocations during parsing.
    pub fn new(xml: Vec<u8>) -> Result<Self, ParseXmlError> {
        // let mut node_count = xml.iter().filter(|&&b| b == b'<').count();
        // let attr_count = xml.iter().filter(|&&b| b == b'=').count();
        let mut node_count = memchr_iter(b'<', xml.as_slice()).count();
        let attr_count = memchr_iter(b'=', xml.as_slice()).count();
        node_count += (node_count / 10) + 1; // Add 10% buffer for nodes

        debug!("Estimated node count: {}", node_count);
        debug!("Estimated attribute count: {}", attr_count);

        let mut doc = Document {
            nodes: Vec::with_capacity(node_count + 1), // +1 for root node
            attributes: Vec::with_capacity(attr_count),
            xml,
        };
        if doc.nodes.capacity() <= node_count || doc.attributes.capacity() < attr_count {
            return Err(ParseXmlError::NotEnoughMemory);
        }
        doc.nodes.push(NodeInfo::new(0, 0, NodeType::Head));
        doc.parse()?;
        doc.nodes.shrink_to_fit();
        doc.attributes.shrink_to_fit();

        warn!(
            "Document created with {} nodes and {} attributes",
            doc.nodes.len(),
            doc.attributes.len()
        );

        warn!(
            "Warning: Expected {} nodes, but found {}",
            node_count,
            doc.nodes.len()
        );

        if attr_count < doc.attributes.len() {
            warn!(
                "Expected {} attributes, but found {}",
                attr_count,
                doc.attributes.len()
            );
        }

        Ok(doc)
    }

    /// Returns the root node of the document.
    pub fn root(&self) -> Option<Node> {
        if self.nodes.len() > 1 {
            Some(Node::new(1, &self.nodes[1], self))
        } else {
            None // No nodes in the document
        }
    }

    /// Retrieves a node by its index.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node to retrieve.
    ///
    /// # Returns
    /// - `Ok(Node)`: The node at the specified index.
    /// - `Err(ParseXmlError)`: If the node index is invalid or out of bounds.
    pub fn get_node(&self, node_idx: NodeIdx) -> Result<Node, ParseXmlError> {
        if node_idx as usize >= self.nodes.len() {
            return Err(ParseXmlError::InvalidXml(format!(
                "Invalid node index: {}",
                node_idx
            )));
        }
        Ok(Node::new(node_idx, &self.nodes[node_idx as usize], self))
    }

    /// Returns the XML content of the document as a byte vector.
    pub fn get_xml_content(&mut self) -> &Vec<u8> {
        &self.xml
    }

    /// Retrieves the parent index of a given node.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node whose parent index is to be retrieved.
    ///
    /// # Returns
    /// - `Ok(NodeIdx)`: The index of the parent node.
    /// - `Err(ParseXmlError)`: If the node index is invalid or if the node has no parent (e.g., root node).
    ///
    /// # Example
    /// ```rust
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let child_node = document.get_node(2).unwrap(); // Assuming 2 is the index of the child node
    /// let parent_idx = document.get_parent_idx(child_node.idx()).unwrap();
    /// assert_eq!(parent_idx, 1); // The parent of the child node is the root node (index 1)
    /// let root_node = document.root().unwrap();
    /// assert_eq!(root_node.idx(), 1); // The root node index is 1
    /// // Attempting to get the parent of the root node should return an error
    /// let parent_of_root = document.get_parent_idx(root_node.idx());
    /// assert!(parent_of_root.is_err(), "Root node should not have a parent");
    /// // Attempting to get the parent of an invalid node index should return an error
    /// let invalid_node_idx = 100; // Assuming this index is out of bounds
    /// let invalid_parent = document.get_parent_idx(invalid_node_idx);
    /// assert!(invalid_parent.is_err(), "Invalid node index should return an error");
    /// ```
    /// # Notes
    /// - The root node (index 1) has no parent, so attempting to get its parent will return an error.
    /// - The method checks if the node index is valid and returns an error if it is not.
    /// # Errors
    /// - `ParseXmlError::InvalidXml`: If the node index is invalid or if the node has no parent (e.g., root node).
    pub fn get_parent_idx(&self, node_idx: NodeIdx) -> Result<NodeIdx, ParseXmlError> {
        if (node_idx == 0) || (node_idx as usize >= self.nodes.len()) {
            return Err(ParseXmlError::InvalidXml(format!(
                "Invalid node index: {}",
                node_idx
            )));
        }

        if node_idx > 1 {
            Ok(self.nodes[node_idx as usize].parent_idx().unwrap())
        } else {
            return Err(ParseXmlError::InvalidXml(
                "Root node has no parent".to_string(),
            ));
        }
    }

    /// Adds a new node to the document.
    /// This method allows adding a new node to the document tree, setting its parent,
    /// and type in the XML source.
    ///
    ///# Arguments
    /// - `parent_idx`: The index of the parent node.
    /// - `node_type`: The type of the node to be added (e.g., element, text).
    ///
    /// # Returns
    /// - `Ok(NodeIdx)`: The index of the newly added node.
    /// - `Err(ParseXmlError)`: If there is an error adding the node (e.g., no more space).
    pub fn add_node(
        &mut self,
        parent_idx: NodeIdx,
        mut node_type: NodeType,
    ) -> Result<NodeIdx, ParseXmlError> {
        let node_idx = self.nodes.len() as NodeIdx;

        if node_idx >= u16::MAX {
            return Err(ParseXmlError::NoMoreSpace);
        }

        if let NodeType::Element { attributes, .. } = &mut node_type {
            *attributes = self.attributes.len() as AttrIdx..self.attributes.len() as AttrIdx;
        }

        let mut node_info = NodeInfo::new(node_idx, parent_idx, node_type);
        let parent_info = &mut self.nodes[parent_idx as usize];

        if parent_info.first_child_idx() == 0 {
            parent_info.set_first_child_idx(node_idx); // Set first child if none exists
            node_info.set_prev_sibling_idx(node_idx); // Set previous sibling to this node (last_child)
        } else {
            let first_child_idx = parent_info.first_child_idx(); // Get first child index of parent
            let last_child_idx = self.nodes[first_child_idx as usize].prev_sibling_idx(); // Get last child Index of parent
            self.nodes[last_child_idx as usize].set_next_sibling_idx(node_idx); // Set next sibling of last child
            self.nodes[first_child_idx as usize].set_prev_sibling_idx(node_idx); // Set previous sibling of first child to last child
            node_info.set_prev_sibling_idx(last_child_idx); // Set previous sibling to last child
        }

        self.nodes.push(node_info);
        Ok(node_idx)
    }

    /// Adds a new attribute to a node.
    /// # Arguments
    /// - `node_idx`: The index of the node to which the attribute will be added.
    /// - `name`: The name of the attribute as a byte range.
    /// - `value`: The value of the attribute as a byte range.
    /// # Returns
    /// - `Ok(AttrIdx)`: The index of the newly added attribute.
    /// - `Err(ParseXmlError)`: If there is an error adding the attribute (e.g., node is not an element).
    pub fn add_attribute(
        &mut self,
        node_idx: NodeIdx,
        name: XmlRange,
        value: XmlRange,
    ) -> Result<AttrIdx, ParseXmlError> {
        let attribute_idx = self.attributes.len() as AttrIdx;
        self.attributes.push(AttributeInfo::new(name, value));
        let node_info = &mut self.nodes[node_idx as usize];
        if !node_info.is_element() {
            return Err(ParseXmlError::InternalError);
        } else {
            let mut attributes_range = match &node_info.node_type() {
                NodeType::Element { attributes, .. } => attributes.clone(),
                _ => return Err(ParseXmlError::InternalError),
            };
            attributes_range.end += 1; // Extend the range to include the new attribute
            node_info.set_node_type(NodeType::Element {
                name: match &node_info.node_type() {
                    NodeType::Element { name, .. } => name.clone(),
                    _ => return Err(ParseXmlError::InternalError),
                },
                attributes: attributes_range,
            });
        }
        Ok(attribute_idx)
    }

    /// Retrieves a string slice from the XML content based on the given range.
    /// # Arguments
    /// - `range`: A reference to an `XmlRange` that specifies the start and end indices of the desired substring.
    /// # Returns
    /// - `&str`: A string slice containing the XML content from the specified range.
    pub fn get_str_from_range<'xml>(&'xml self, range: &XmlRange) -> &'xml str {
        let xml_content = &self.xml[range.start as usize..range.end as usize];
        std::str::from_utf8(xml_content).unwrap_or("non valid utf-8")
    }
}

impl<'input> fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(root) = self.root() {
            // write!(f, "Document [{}]", root.tag_name())?;

            macro_rules! writeln_indented {
                ($indent:expr, $f:expr, $fmt:expr) => {
                    for _ in 0..$indent { write!($f, "    ")?; }
                    writeln!($f, $fmt)?;
                };

                ($indent:expr, $f:expr, $fmt:expr, $($arg:tt)*) => {
                    for _ in 0..$indent { write!($f, "    ")?; }
                    writeln!($f, $fmt, $($arg)*)?;
                };
            }

            fn print_into_iter<
                T: fmt::Debug,
                E: ExactSizeIterator<Item = T>,
                I: IntoIterator<Item = T, IntoIter = E>,
            >(
                prefix: &str,
                data: I,
                indent: usize,
                f: &mut fmt::Formatter,
            ) -> Result<(), fmt::Error> {
                let data = data.into_iter();

                if data.len() == 0 {
                    return Ok(());
                }

                writeln_indented!(indent, f, "{}: [", prefix);
                for v in data {
                    writeln_indented!(indent + 1, f, "{:?}", v);
                }
                writeln_indented!(indent, f, "]");
                Ok(())
            }

            fn print_node(
                node: Node,
                indent: usize,
                f: &mut fmt::Formatter,
            ) -> Result<(), fmt::Error> {
                if node.is_element() {
                    writeln_indented!(indent, f, "Element {{");
                    writeln_indented!(indent, f, "    tag_name: {:?}", node.tag_name());
                    print_into_iter("attributes", node.attributes(), indent + 1, f)?;

                    if node.has_children() {
                        writeln_indented!(indent, f, "    children: [");
                        print_children(node, indent + 2, f)?;
                        writeln_indented!(indent, f, "    ]");
                    }

                    writeln_indented!(indent, f, "}}");
                } else if node.is_text() {
                    writeln_indented!(indent, f, "Text {{");
                    writeln_indented!(indent, f, "    \"{}\"", node.text().unwrap_or("No text"));
                    writeln_indented!(indent, f, "}}");
                    //writeln_indented!(indent, f, "{:?}", node);
                    // } else if node.is_root() {
                    //     writeln_indented!(indent, f, "Root {{}}");
                } else {
                    writeln_indented!(indent, f, "Unknown Node!");
                }
                Ok(())
            }

            fn print_children(
                parent: Node,
                indent: usize,
                f: &mut fmt::Formatter,
            ) -> Result<(), fmt::Error> {
                for child in parent.children() {
                    print_node(child, indent, f)?;
                }

                Ok(())
            }

            writeln!(f, "Document [")?;
            print_node(root, 1, f)?;
            writeln!(f, "]")?;

            Ok(())
        } else {
            write!(f, "Document [No root node]")?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let xml_data = b"<root><child>Text</child><totototo/></root>".to_vec();
        let mut document = Document::new(xml_data).unwrap();

        println!("Document created: {:#?}", document);
    }
}
