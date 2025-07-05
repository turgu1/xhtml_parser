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
#[must_use]
#[derive(Debug, Clone)]
pub struct Node<'xml> {
    pub idx: NodeIdx,
    #[cfg(feature = "forward_only")]
    pub parent_idx: NodeIdx,
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
    #[inline]
    pub(crate) fn new(
        idx: NodeIdx,
        #[cfg(feature = "forward_only")] // Only used in forward-only mode
        parent_idx: NodeIdx,
        node_info: &'xml NodeInfo,
        doc: &'xml Document,
    ) -> Self {
        Node {
            idx,
            #[cfg(feature = "forward_only")]
            parent_idx,
            node_info,
            doc,
        }
    }

    /// Returns the index of the node in the document.
    #[inline]
    #[must_use]
    pub fn idx(&self) -> NodeIdx {
        self.idx
    }

    /// Returns the index of the parent node, if it exists.
    #[inline]
    #[must_use]
    pub(crate) fn parent_idx(&self) -> Option<NodeIdx> {
        if self.idx <= 1 {
            None // The root node has no parent
        } else {
            #[cfg(feature = "forward_only")]
            if self.parent_idx != 0 {
                return Some(self.parent_idx);
            } else {
                return None; // In forward-only mode, the parent index may not stored in the node info
            }
            #[cfg(not(feature = "forward_only"))]
            // In non-forward-only mode, the parent index is stored in the node info
            return self.node_info.parent_idx();
        }
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
    #[inline]
    #[must_use]
    pub fn tag_name(&self) -> &str {
        match &self.node_info.node_type() {
            NodeType::Element { name, .. } => self.doc.get_str_from_location(name.clone()),
            _ => "", // No tag name for non-element nodes
        }
    }

    /// Returns true if the node's tag name matches the provided tag name, false otherwise.
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
    pub fn attributes(&self) -> Attributes<'xml> {
        Attributes::new(self)
    }

    /// Returns the first child index of the node, if it exists, None otherwise.
    ///
    /// If the node has no children, it returns None.
    /// If the node is in forward-only mode, it returns the next index that is not a sibling of the current node.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    ///
    /// let xml_data = b"<root><child1/><child2/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let first_child_idx = root_node.first_child_idx();
    ///
    /// assert_eq!(first_child_idx, Some(2)); // Assuming the first child is at index 2
    /// ```
    #[inline]
    #[must_use]
    pub fn first_child_idx(&self) -> Option<NodeIdx> {
        #[cfg(not(feature = "forward_only"))]
        {
            if self.node_info.first_child_idx() == 0 {
                None
            } else {
                Some(self.node_info.first_child_idx())
            }
        }

        #[cfg(feature = "forward_only")]
        // In forward-only mode, the first child is the next index not sibling of the current node
        if (self.node_info.next_sibling_idx() == self.idx + 1)
            || (self.doc.last_node_idx() == self.idx)
        {
            None
        } else {
            Some(self.idx + 1)
        }
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
    #[inline]
    #[must_use]
    pub fn first_child(&self) -> Option<Node<'xml>> {
        self.first_child_idx().map(|first_child_idx| {
            Node::new(
                first_child_idx,
                #[cfg(feature = "forward_only")]
                self.idx,
                &self.doc.nodes[first_child_idx as usize],
                self.doc,
            )
        })
    }

    #[cfg(not(feature = "forward_only"))]
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
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn next_sibling(&self) -> Option<Node<'xml>> {
        if self.node_info.next_sibling_idx() == 0 {
            None
        } else {
            Some(Node::new(
                self.node_info.next_sibling_idx(),
                #[cfg(feature = "forward_only")]
                self.parent_idx,
                &self.doc.nodes[self.node_info.next_sibling_idx() as usize],
                self.doc,
            ))
        }
    }

    #[cfg(not(feature = "forward_only"))]
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
    #[must_use]
    pub fn prev_sibling(&self) -> Option<Node<'xml>> {
        let node_info = &self.doc.nodes[self.node_info.prev_sibling_idx() as usize];
        if node_info.next_sibling_idx() == 0 {
            None // this is the last child... not the previous sibling
        } else {
            Some(Node::new(
                self.node_info.prev_sibling_idx(),
                #[cfg(feature = "forward_only")]
                self.parent_idx,
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
    #[inline]
    #[must_use]
    pub fn children(&self) -> NodeChildren<'xml> {
        if self.has_children() {
            #[cfg(not(feature = "forward_only"))]
            {
                NodeChildren {
                    front: self.first_child(),
                    back: self.last_child(),
                }
            }
            #[cfg(feature = "forward_only")]
            {
                NodeChildren {
                    front: self.first_child(),
                    back: None,
                }
            }
        } else {
            NodeChildren {
                front: None,
                back: None,
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
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.first_child_idx().is_some()
    }

    /// Returns true if the node is a `NodeType::Element`, false otherwise.
    #[inline]
    #[must_use]
    pub fn is_element(&self) -> bool {
        matches!(self.node_info.node_type(), NodeType::Element { .. })
    }

    /// Returns true if the node is a `NodeType::Text`, false otherwise.
    #[inline]
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self.node_info.node_type(), NodeType::Text(_))
    }

    /// Returns the `NodeType` instance associated with this node.
    #[inline]
    #[must_use]
    pub fn get_node_type(&self) -> &NodeType {
        self.node_info.node_type()
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
    #[must_use]
    pub fn get_child(&self, tag_name: &str) -> Option<Node<'xml>> {
        self.children().find(|child| child.is(tag_name))

        // self.first_child_idx()
        //     .map(|first_child_idx| {
        //         let mut current_idx = first_child_idx;
        //         loop {
        //             let current_node_info = &self.doc.nodes[current_idx as usize];

        //             #[cfg(not(feature = "forward_only"))]
        //             let current_node = Node::new(current_idx, current_node_info, self.doc);

        //             #[cfg(feature = "forward_only")]
        //             let current_node =
        //                 Node::new(current_idx, self.idx, current_node_info, self.doc);

        //             if current_node.is(tag_name) {
        //                 return Some(current_node);
        //             }

        //             if current_node_info.next_sibling_idx() == 0 {
        //                 break;
        //             }
        //             current_idx = current_node_info.next_sibling_idx();
        //         }
        //         None
        //     })
        //     .flatten()

        // let mut current_idx = self.node_info.first_child_idx();
        // loop {
        //     let current_node_info = &self.doc.nodes[current_idx as usize];
        //     let current_node = Node::new(current_idx, current_node_info, self.doc);

        //     if current_node.is(tag_name) {
        //         return Some(current_node);
        //     }

        //     if current_node_info.next_sibling_idx() == 0 {
        //         break;
        //     }
        //     current_idx = current_node_info.next_sibling_idx();
        // }
        // None
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
    #[must_use]
    pub fn get_sibling(&self, tag_name: &str) -> Option<Node<'xml>> {
        self.parent()
            .map(|parent| parent.children().find(|sibling| sibling.is(tag_name)))
            .flatten()

        // self.parent_idx()
        //     .map(|parent_idx| {
        //         let parent_node_info = &self.doc.nodes[parent_idx as usize];
        //         if parent_node_info.first_child_idx() == 0 {
        //             return None;
        //         }

        //         let mut current_idx = parent_node_info.first_child_idx();
        //         loop {
        //             let current_node_info = &self.doc.nodes[current_idx as usize];
        //             let current_node = Node::new(current_idx, current_node_info, self.doc);

        //             if current_node.is(tag_name) {
        //                 return Some(current_node);
        //             }

        //             if current_node_info.next_sibling_idx() == 0 {
        //                 break;
        //             }
        //             current_idx = current_node_info.next_sibling_idx();
        //         }
        //         None
        //     })
        //     .flatten()
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
    #[must_use]
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
    #[must_use]
    pub fn parent(&self) -> Option<Node<'xml>> {
        #[cfg(not(feature = "forward_only"))]
        return self.parent_idx().map(|parent_idx| {
            Node::new(parent_idx, &self.doc.nodes[parent_idx as usize], self.doc)
        });
        #[cfg(feature = "forward_only")]
        return self.parent_idx().map(|parent_idx| {
            Node::new(
                parent_idx,
                0, // In forward-only mode, parent_idx of parent is 0 as we can't traverse backwards
                &self.doc.nodes[parent_idx as usize],
                self.doc,
            )
        });
    }

    /// Returns the position of this node in the XML source.
    #[inline]
    #[must_use]
    pub fn position(&self) -> XmlIdx {
        self.node_info.position()
    }
}

impl Eq for Node<'_> {}

impl PartialEq for Node<'_> {
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

#[cfg(not(feature = "forward_only"))]
impl DoubleEndedIterator for NodeChildren<'_> {
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
