//! Node representation and manipulation in an XML document.
//!
//! This module defines the `Node` struct, which represents a node in an XML document.
//! It provides methods to access node properties, navigate the document tree, and retrieve attributes and text content.
//!
//! The `Node` struct is designed to work with a `Document`, which contains the XML data and node information.
//!
//! # Example
//!
//! ```
//! use xhtml_parser::Document;
//! use xhtml_parser::Node;
//!
//! let xml_data = b"<root><child>Text</child></root>".to_vec();
//! let document = Document::new(xml_data).unwrap();
//! let root_node = document.root().unwrap();
//! let child_node = root_node.first_child().unwrap();
//!
//! assert!(child_node.is("child"));
//!
//! let child_node = child_node.first_child().unwrap();
//!
//! assert_eq!(child_node.text().unwrap(), "Text");
//! assert!(!child_node.is_element());
//! assert!(child_node.is_text());
//! ```
//!
//! # Features
//!
//! - Access to node attributes and namespaces
//! - Navigation through the document tree (children, siblings)
//! - Support for different node types (elements, text, etc.)
//! - Iteration over node children
//!
//! # Note
//! This module is part of the `xhtml_parser` crate and is designed to work with XML documents.

use crate::attribute::Attributes;
use crate::defs::{NodeIdx, XmlIdx};
use crate::document::{Document, Nodes};
use crate::node_info::NodeInfo;
use crate::node_type::NodeType;

/// Represents a node in an XML document.
///
/// `Node` contains metadata about the node, such as its index, type, and position in the document.
/// It provides methods to access the node's tag name, text content, attributes, and navigation through the document tree.
#[derive(Debug, Clone)]
pub struct Node<'xml> {
    pub idx: NodeIdx,
    pub node_info: &'xml NodeInfo,
    pub doc: &'xml Document,
}

impl<'xml> Node<'xml> {
    /// Creates a new `Node` instance.
    ///
    /// # Arguments
    /// - `idx`: The index of the node in the document.
    /// - `node_info`: A reference to the `NodeInfo` containing metadata about the node.
    /// - `doc`: A reference to the `Document` containing the XML data.
    pub(crate) fn new(idx: NodeIdx, node_info: &'xml NodeInfo, doc: &'xml Document) -> Self {
        Node {
            idx,
            node_info,
            doc,
        }
    }

    /// Returns the index of the node in the document.
    pub fn idx(&self) -> NodeIdx {
        self.idx
    }

    /// Returns the tag name of the node.
    /// If the node is not an element, it returns an empty string.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let tag_name = root_node.tag_name();
    ///
    /// assert_eq!(tag_name, "root");
    /// ```
    pub fn tag_name(&self) -> &str {
        match &self.node_info.node_type() {
            NodeType::Element { name, .. } => self.doc.get_str_from_location(name.clone()),
            _ => "", // No tag name for non-element nodes
        }
    }

    /// Returns true if the node's tag name matches the provided tag name, false otherwise.
    #[inline]
    pub fn is(&self, tag_name: &str) -> bool {
        self.tag_name() == tag_name
    }

    /// Returns the text content of the node.
    /// If the node is not a text node, it returns an empty string.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root>The Text</root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let child_node = root_node.first_child().unwrap();
    ///
    /// assert!(child_node.is_text());
    ///
    /// let text_content = child_node.text().unwrap();
    ///
    /// assert_eq!(text_content, "The Text");
    /// ```
    pub fn text(&self) -> Option<&'xml str> {
        match &self.node_info.node_type() {
            NodeType::Text(text_location) => {
                Some(self.doc.get_str_from_location(text_location.clone()))
            }
            _ => None,
        }
    }

    /// Returns a new `Attributes` iterator instance for this node.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root name=\"The root\" id=\"1\">Text</root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let attributes: Vec<_> = root_node.attributes().collect();
    ///
    /// assert_eq!(attributes.len(), 2);
    /// assert_eq!(attributes[0].name(), "name");
    /// assert_eq!(attributes[0].value(), "The root");
    /// assert_eq!(attributes[1].name(), "id");
    /// assert_eq!(attributes[1].value(), "1");
    /// ```
    pub fn attributes(&self) -> Attributes<'xml> {
        Attributes::new(self)
    }

    /// Returns the first child of the node, if it exists, None otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let node = document.root().unwrap();    
    ///
    /// assert!(node.first_child().unwrap().is("child1"));
    /// ```
    pub fn first_child(&self) -> Option<Node<'xml>> {
        if self.node_info.first_child_idx() == 0 {
            None
        } else {
            Some(Node::new(
                self.node_info.first_child_idx(),
                &self.doc.nodes[self.node_info.first_child_idx() as usize],
                self.doc,
            ))
        }
    }

    /// Returns the last child of the node, if it exists, None otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let last_child = root_node.last_child().unwrap();
    ///
    /// assert!(last_child.is("child2"));
    /// ```
    pub fn last_child(&self) -> Option<Node<'xml>> {
        if self.node_info.first_child_idx() == 0 {
            None
        } else {
            let first_child_idx = self.node_info.first_child_idx();
            let last_child_idx = self.doc.nodes[first_child_idx as usize].prev_sibling_idx();
            Some(Node::new(
                last_child_idx,
                &self.doc.nodes[last_child_idx as usize],
                self.doc,
            ))
        }
    }

    /// Returns the next sibling of the node, if it exists, None otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let next_sibling = root_node.first_child().unwrap().next_sibling().unwrap();
    ///
    /// assert!(next_sibling.is("child2"));
    /// ```
    #[inline]
    pub fn next_sibling(&self) -> Option<Node<'xml>> {
        if self.node_info.next_sibling_idx() == 0 {
            None
        } else {
            Some(Node::new(
                self.node_info.next_sibling_idx(),
                &self.doc.nodes[self.node_info.next_sibling_idx() as usize],
                self.doc,
            ))
        }
    }

    /// Returns the previous sibling of the node, if it exists, None otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let prev_sibling = root_node.last_child().unwrap().prev_sibling().unwrap();
    ///
    /// assert!(prev_sibling.is("child1"));
    /// ```
    #[inline]
    pub fn prev_sibling(&self) -> Option<Node<'xml>> {
        let node_info = &self.doc.nodes[self.node_info.prev_sibling_idx() as usize];
        if node_info.next_sibling_idx() == 0 {
            None // this is the last child... not the previous sibling
        } else {
            Some(Node::new(
                self.node_info.prev_sibling_idx(),
                node_info,
                self.doc,
            ))
        }
    }

    /// Returns an iterator over the children of the node.
    /// If the node has no children, it returns an empty iterator.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let children: Vec<_> = root_node.children().collect();
    ///
    /// assert_eq!(children.len(), 2);
    /// assert!(children[0].is("child1"));
    /// assert!(children[1].is("child2"));
    /// ```
    pub fn children(&self) -> NodeChildren {
        if self.node_info.first_child_idx() == 0 {
            NodeChildren {
                front: None,
                back: None,
            }
        } else {
            NodeChildren {
                front: self.first_child(),
                back: self.last_child(),
            }
        }
    }

    /// Returns an iterator over all descendants of the node.
    ///
    /// This includes all children, grandchildren, and so on.
    /// If the node has no descendants, it returns an empty iterator.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1><subchild/></child1><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let descendants: Vec<_> = root_node.descendants().collect();
    /// ```
    pub fn descendants(&self) -> Nodes<'xml> {
        Nodes::descendants(self.doc, self.idx)
    }

    /// Returns true if the node is the root node, false otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    ///
    /// assert!(root_node.is_root());
    /// ```
    pub fn is_root(&self) -> bool {
        self.idx == 1 // The root node is always at index 1
    }

    /// Returns true if the node has children, false otherwise.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    ///
    /// assert!(root_node.has_children());
    /// ```
    #[inline]
    pub fn has_children(&self) -> bool {
        self.node_info.first_child_idx() != 0
    }

    /// Returns true if the node is a NodeType::Element, false otherwise.
    #[inline]
    pub fn is_element(&self) -> bool {
        matches!(self.node_info.node_type(), NodeType::Element { .. })
    }

    /// Returns true if the node is a NodeType::Text, false otherwise.
    #[inline]
    pub fn is_text(&self) -> bool {
        matches!(self.node_info.node_type(), NodeType::Text(_))
    }

    /// Returns the NodeType instance associated with this node.
    #[inline]
    pub fn get_node_type(&self) -> &NodeType {
        &self.node_info.node_type()
    }

    /// Finds a child node with the specified tag name.
    /// If the node has no children, it returns None.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    ///
    /// if let Some(child) = root_node.get_child("child2") {
    ///     assert!(child.is("child2"));
    /// } else {
    ///     panic!("Child node not found");
    /// }
    /// ```
    pub fn get_child(&self, tag_name: &str) -> Option<Node<'xml>> {
        if self.node_info.first_child_idx() == 0 {
            return None;
        }

        let mut current_idx = self.node_info.first_child_idx();
        loop {
            let current_node_info = &self.doc.nodes[current_idx as usize];
            let current_node = Node::new(current_idx, current_node_info, self.doc);

            if current_node.is(tag_name) {
                return Some(current_node);
            }

            if current_node_info.next_sibling_idx() == 0 {
                break;
            }
            current_idx = current_node_info.next_sibling_idx();
        }
        None
    }

    /// Finds a sibling node with the specified tag name.
    /// If the node has no parent or no siblings, it returns None.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let child_node = root_node.first_child().unwrap();
    ///
    /// if let Some(sibling) = child_node.get_sibling("child2") {
    ///     assert!(sibling.is("child2"));
    /// } else {
    ///     panic!("Sibling node not found");
    /// }
    /// ```
    pub fn get_sibling(&self, tag_name: &str) -> Option<Node<'xml>> {
        if let Some(parent_idx) = self.node_info.parent_idx() {
            let parent_node_info = &self.doc.nodes[parent_idx as usize];
            if parent_node_info.first_child_idx() == 0 {
                return None;
            }

            let mut current_idx = parent_node_info.first_child_idx();
            loop {
                let current_node_info = &self.doc.nodes[current_idx as usize];
                let current_node = Node::new(current_idx, current_node_info, self.doc);

                if current_node.is(tag_name) {
                    return Some(current_node);
                }

                if current_node_info.next_sibling_idx() == 0 {
                    break;
                }
                current_idx = current_node_info.next_sibling_idx();
            }
            None
        } else {
            None
        }
    }

    /// searches for an attribute by name and returns its value if found.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root name=\"value\">Text</root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    ///
    /// if let Some(value) = root_node.get_attribute("name") {
    ///     assert_eq!(value, "value");
    /// } else {
    ///     panic!("Attribute not found");
    /// }
    /// ```
    #[inline]
    pub fn get_attribute(&self, name: &str) -> Option<&'xml str> {
        for attr in self.attributes() {
            if attr.name() == name {
                return Some(attr.value());
            }
        }
        None
    }

    /// Returns the parent node of this node, if it exists.
    /// If this node is the root node, it returns None.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let child_node = root_node.first_child().unwrap();
    ///
    /// if let Some(parent) = child_node.parent() {
    ///     assert!(parent.is("root"));
    /// } else {
    ///     panic!("Child node has no parent");
    /// }
    /// ```
    #[inline]
    pub fn parent(&self) -> Option<Node<'xml>> {
        if let Some(parent_idx) = self.node_info.parent_idx() {
            Some(Node::new(
                parent_idx,
                &self.doc.nodes[parent_idx as usize],
                self.doc,
            ))
        } else {
            None // Root node has no parent
        }
    }

    /// Returns the position of this node in the XML source.
    #[inline]
    pub fn position(&self) -> XmlIdx {
        self.node_info.position()
    }
}

impl<'xml> Eq for Node<'xml> {}

impl<'xml> PartialEq for Node<'xml> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

/// Iterator over node children.
///
/// This iterator allows traversing the children of a node in both forward and backward directions.
/// It is designed to work with the `Node` struct, providing an easy way to access child nodes sequentially.
///
/// # Example
///
/// ```
/// use xhtml_parser::Document;
///
/// let xml_data = b"<root><child1/><child2/></root>".to_vec();
/// let document = Document::new(xml_data).unwrap();
/// let root_node = document.root().unwrap();
/// let children: Vec<_> = root_node.children().collect();
///
/// assert_eq!(children.len(), 2);
/// assert!(children[0].is("child1"));
/// assert!(children[1].is("child2"));
/// ```
pub struct NodeChildren<'a> {
    front: Option<Node<'a>>,
    back: Option<Node<'a>>,
}

impl<'a> Iterator for NodeChildren<'a> {
    type Item = Node<'a>;

    /// Returns the next child node in the iteration.
    ///
    /// If there are no more children, it returns None.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            let node = self.front.take();
            self.back = None;
            node
        } else {
            let node = self.front.take();
            self.front = node.as_ref().and_then(Node::next_sibling);
            node
        }
    }
}

impl<'a> DoubleEndedIterator for NodeChildren<'a> {
    /// Returns the previous child node in the iteration.
    ///
    /// If there are no more children, it returns None.
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            let node = self.back.take();
            self.front = None;
            node
        } else {
            let node = self.back.take();
            self.back = node.as_ref().and_then(Node::prev_sibling);
            node
        }
    }
}
