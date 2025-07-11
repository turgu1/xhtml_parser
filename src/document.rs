//! Document module for the `xhtml-parser` crate.
//!
//!

#![allow(clippy::cast_possible_truncation)]

use log::{debug, warn};

use memchr::memchr_iter;
use std::fmt::{self};

use crate::attribute::AttributeInfo;
use crate::defs::{AttrIdx, NodeIdx, ParseXmlError, XmlIdx, XmlLocation};
use crate::node::Node;
use crate::node_info::NodeInfo;
use crate::node_type::NodeType;

/// Represents a parsed XML document.
///
/// The `Document` struct contains a vector of `NodeInfo` representing the nodes in the document,
/// a vector of `AttributeInfo` representing the attributes, and the raw XML content as a byte vector.
/// It provides methods to create a new document from XML content, retrieve the root node,
/// get nodes by index, add new nodes and attributes, and access the XML content.

#[derive(PartialEq, Eq)]
#[must_use]
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
    /// # Errors
    /// - `ParseXmlError::InvalidXml`: If the XML content is not well-formed or contains errors.
    /// - `ParseXmlError::NoMoreSpace`: If there is not enough space to add new nodes or attributes.
    /// - `ParseXmlError::NotEnoughMemory`: If there is not enough memory to allocate the document's nodes or attributes.
    ///
    /// # Example
    /// ```
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    ///
    /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root = document.root().unwrap();
    ///
    /// assert_eq!(root.tag_name(), "root");
    ///
    /// let child = root.first_child().unwrap();
    ///
    /// assert_eq!(child.tag_name(), "child");
    ///
    /// let child_text = child.first_child().unwrap();
    ///
    /// assert_eq!(child_text.text().unwrap(), "Text");
    /// ```
    /// # Notes
    /// - The `Document` struct is designed to handle XML documents and provides methods for navigating the document tree.
    /// - The `new` method estimates the number of nodes and attributes based on the XML content and allocates memory accordingly.
    ///   This is done to optimize performance and reduce memory reallocations during parsing.
    pub fn new(xml: Vec<u8>) -> Result<Self, ParseXmlError> {
        let mut node_count = memchr_iter(b'<', xml.as_slice()).count();
        let attr_count = memchr_iter(b'=', xml.as_slice()).count();
        node_count += (node_count / 10) + 1; // Add 10% buffer for nodes

        debug!("Estimated node count: {node_count}");
        debug!("Estimated attribute count: {attr_count}");

        if node_count > NodeIdx::MAX as usize {
            return Err(ParseXmlError::InvalidXml(
                "XML document has too many estimated nodes!".to_string(),
            ));
        }

        if attr_count > AttrIdx::MAX as usize {
            return Err(ParseXmlError::InvalidXml(
                "XML document has too many estimated attributes!".to_string(),
            ));
        }

        if xml.len() > XmlIdx::MAX as usize {
            return Err(ParseXmlError::InvalidXml(
                "XML document is too large!".to_string(),
            ));
        }

        let mut doc = Document {
            nodes: Vec::with_capacity(node_count + 1), // +1 for root node
            attributes: Vec::with_capacity(attr_count),
            xml,
        };
        if doc.nodes.capacity() <= node_count || doc.attributes.capacity() < attr_count {
            return Err(ParseXmlError::NotEnoughMemory);
        }

        // Add the head node as the first node in the document.
        #[cfg(not(feature = "forward_only"))]
        doc.nodes.push(NodeInfo::new(0, 0, NodeType::Head));
        #[cfg(feature = "forward_only")]
        doc.nodes.push(NodeInfo::new(NodeType::Head));

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
    #[inline]
    #[must_use]
    pub fn root(&self) -> Option<Node<'_>> {
        #[cfg(not(feature = "forward_only"))]
        if self.nodes.len() > 1 {
            Some(Node::new(1, &self.nodes[1], self))
        } else {
            None // No nodes in the document
        }

        #[cfg(feature = "forward_only")]
        if self.nodes.len() > 1 {
            Some(Node::new(1, 0, &self.nodes[1], self))
        } else {
            None // No nodes in the document
        }
    }

    /// Checks if the document is empty.
    ///
    /// # Returns
    /// - `true`: If the document contains only the head node (no other nodes).
    /// - `false`: If the document contains nodes other than the head node.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1 // Only the head node exists
    }

    /// Returns the index of the last node in the document.
    ///
    /// # Returns
    /// - `NodeIdx`: The index of the last node in the document.
    /// - `0`: If the document is empty (no nodes).
    #[inline]
    #[must_use]
    pub fn last_node_idx(&self) -> NodeIdx {
        if self.is_empty() {
            0 // No nodes, return 0
        } else {
            (self.nodes.len() - 1) as NodeIdx // Last node index
        }
    }

    #[cfg(not(feature = "forward_only"))]
    /// Retrieves a node by its index.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node to retrieve.
    ///
    /// # Returns
    /// - `Ok(Node)`: The node at the specified index.
    /// - `Err(ParseXmlError)`: If the node index is invalid or out of bounds.
    ///
    /// # Errors
    /// - `ParseXmlError::InvalidXml`: If the node index is invalid or out of bounds.
    #[inline]
    pub fn get_node(&self, node_idx: NodeIdx) -> Result<Node<'_>, ParseXmlError> {
        if node_idx as usize >= self.nodes.len() {
            return Err(ParseXmlError::InvalidXml(format!(
                "Invalid node index: {node_idx}"
            )));
        }
        Ok(Node::new(node_idx, &self.nodes[node_idx as usize], self))
    }

    #[cfg(feature = "forward_only")]
    /// Retrieves a node by its index.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node to retrieve.
    ///
    /// # Returns
    /// - `Ok(Node)`: The node at the specified index.
    /// - `Err(ParseXmlError)`: If the node index is invalid or out of bounds.
    ///
    /// # Errors
    /// - `ParseXmlError::InvalidXml`: If the node index is invalid or out of bounds.
    ///
    /// # Notes
    /// This method is optimized for forward-only traversal of nodes. It is **not** suitable to
    /// get access to the parent node of any previous node.
    /// It is designed to be used in scenarios where nodes are processed in a forward-only manner
    #[inline]
    pub fn get_node(&self, node_idx: NodeIdx) -> Result<Node<'_>, ParseXmlError> {
        if node_idx as usize >= self.nodes.len() {
            return Err(ParseXmlError::InvalidXml(format!(
                "Invalid node index: {node_idx}"
            )));
        }
        Ok(Node::new(node_idx, 0, &self.nodes[node_idx as usize], self))
    }

    /// Returns the XML content of the document as a byte vector.
    #[inline]
    #[must_use]
    pub fn get_xml_content(&mut self) -> &Vec<u8> {
        &self.xml
    }

    // No longer needed. I keep the code in case it would be required again
    // --------------------------------------------------------------------
    //
    // /// Retrieves the parent index of a given node.
    // ///
    // /// # Arguments
    // /// - `node_idx`: The index of the node whose parent index is to be retrieved.
    // ///
    // /// # Returns
    // /// - `Ok(NodeIdx)`: The index of the parent node.
    // /// - `Err(ParseXmlError)`: If the node index is invalid or if the node has no parent (e.g., root node).
    // ///
    // /// # Example
    // /// ```rust
    // /// use xhtml_parser::Document;
    // ///
    // /// let xml_data = b"<root><child>Text</child></root>".to_vec();
    // /// let document = Document::new(xml_data).unwrap();
    // /// let child_node = document.get_node(2).unwrap(); // Assuming 2 is the index of the child node
    // /// let parent_idx = document.get_parent_idx(child_node.idx()).unwrap();
    // ///
    // /// assert_eq!(parent_idx, 1); // The parent of the child node is the root node (index 1)
    // ///
    // /// let root_node = document.root().unwrap();
    // ///
    // /// assert_eq!(root_node.idx(), 1); // The root node index is 1
    // ///
    // /// // Attempting to get the parent of the root node should return an error
    // /// let parent_of_root = document.get_parent_idx(root_node.idx());
    // ///
    // /// assert!(parent_of_root.is_err(), "Root node should not have a parent");
    // ///
    // /// // Attempting to get the parent of an invalid node index should return an error
    // /// let invalid_node_idx = 100; // Assuming this index is out of bounds
    // /// let invalid_parent = document.get_parent_idx(invalid_node_idx);
    // ///
    // /// assert!(invalid_parent.is_err(), "Invalid node index should return an error");
    // /// ```
    // /// # Notes
    // /// - The root node (index 1) has no parent, so attempting to get its parent will return an error.
    // /// - The method checks if the node index is valid and returns an error if it is not.
    // /// # Errors
    // /// - `ParseXmlError::InvalidXml`: If the node index is invalid or if the node has no parent (e.g., root node).
    // pub fn get_parent_idx(&self, node_idx: NodeIdx) -> Result<NodeIdx, ParseXmlError> {
    //     if (node_idx == 0) || (node_idx as usize >= self.nodes.len()) {
    //         Err(ParseXmlError::InvalidXml(format!(
    //             "Invalid node index: {node_idx}"
    //         )))
    //     } else if node_idx > 1 {
    //         self.nodes[node_idx as usize].parent_idx().ok_or_else(|| {
    //             ParseXmlError::InvalidXml(format!("Node index {node_idx} has no parent"))
    //         })
    //     } else {
    //         Err(ParseXmlError::InvalidXml(
    //             "Root node has no parent".to_string(),
    //         ))
    //     }
    // }

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
    pub(crate) fn add_node(
        &mut self,
        parent_idx: NodeIdx,
        last_child_idx: NodeIdx,
        mut node_type: NodeType,
    ) -> Result<NodeIdx, ParseXmlError> {
        let node_idx = self.nodes.len() as NodeIdx;

        if node_idx == NodeIdx::MAX {
            return Err(ParseXmlError::NoMoreSpace);
        }

        if let NodeType::Element { attributes, .. } = &mut node_type {
            *attributes = self.attributes.len() as AttrIdx..self.attributes.len() as AttrIdx;
        }

        #[cfg(not(feature = "forward_only"))]
        let mut node_info = NodeInfo::new(node_idx, parent_idx, node_type);

        #[cfg(feature = "forward_only")]
        let node_info = NodeInfo::new(node_type);

        #[cfg(not(feature = "forward_only"))]
        {
            let parent_info = &mut self.nodes[parent_idx as usize];

            if parent_info.first_child_idx() == 0 {
                parent_info.set_first_child_idx(node_idx); // Set first child if none exists
                node_info.set_prev_sibling_idx(node_idx); // Set previous sibling to this node (last_child)
            } else {
                let first_child_idx = parent_info.first_child_idx(); // Get first child index of parent

                // let last_child_idx = self.nodes[first_child_idx as usize].prev_sibling_idx(); // Get last child Index of parent
                self.nodes[last_child_idx as usize].set_next_sibling_idx(node_idx); // Set next sibling of last child
                self.nodes[first_child_idx as usize].set_prev_sibling_idx(node_idx); // Set previous sibling of first child to last child
                node_info.set_prev_sibling_idx(last_child_idx); // Set previous sibling to last child
            }
        }

        #[cfg(feature = "forward_only")]
        {
            let parent_info = &mut self.nodes[parent_idx as usize];

            if parent_info.first_child_idx() == 0 {
                parent_info.set_first_child_idx(node_idx); // Set first child if none exists
            } else if last_child_idx != 0 {
                // If there is a last child, set the next sibling index of the last child to this node
                self.nodes[last_child_idx as usize].set_next_sibling_idx(node_idx);
            }
        }

        // if parent_idx != self.last_node_idx() {
        //     self.nodes[self.last_node_idx() as usize].set_next_sibling_idx(self.nodes.len() as NodeIdx);
        // }
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
    pub(crate) fn add_attribute(
        &mut self,
        node_idx: NodeIdx,
        name: XmlLocation,
        value: XmlLocation,
    ) -> Result<AttrIdx, ParseXmlError> {
        let attribute_idx = self.attributes.len() as AttrIdx;
        self.attributes.push(AttributeInfo::new(name, value));
        let node_info = &mut self.nodes[node_idx as usize];

        if !node_info.is_element() {
            return Err(ParseXmlError::InternalError);
        }

        let mut attributes_range = match &node_info.node_type() {
            NodeType::Element { attributes, .. } => attributes.clone(),
            _ => return Err(ParseXmlError::InternalError),
        };
        attributes_range.end += 1; // Extend the range to include the new attribute
        node_info.set_node_type(NodeType::Element {
            name: match &node_info.node_type() {
                #[cfg(not(feature = "use_cstr"))]
                NodeType::Element { name, .. } => name.clone(),

                #[cfg(feature = "use_cstr")]
                NodeType::Element { name, .. } => *name,

                _ => return Err(ParseXmlError::InternalError),
            },
            attributes: attributes_range,
        });

        Ok(attribute_idx)
    }

    /// Retrieves a string slice from the XML content based on the given range.
    /// # Arguments
    /// - `range`: A reference to an `XmlLocation` that specifies the start and end indices of the desired substring.
    /// # Returns
    /// - `&str`: A string slice containing the XML content from the specified range.
    #[inline]
    #[must_use]
    pub fn get_str_from_location(&self, location: XmlLocation) -> &str {
        #[cfg(not(feature = "use_cstr"))]
        {
            let xml_content = &self.xml[location.start as usize..location.end as usize];
            std::str::from_utf8(xml_content).unwrap_or("non valid utf-8")
        }

        #[cfg(feature = "use_cstr")]
        {
            let content = std::ffi::CStr::from_bytes_until_nul(&self.xml[location as usize..])
                .unwrap_or(c"cstr not valid");
            content.to_str().unwrap_or("non valid utf-8")
        }
    }

    /// Returns an iterator over all nodes in the document.
    ///
    /// This method provides an iterator that traverses all nodes in the document, starting from the root node.
    ///
    /// # Returns
    /// - `Nodes`: An iterator that yields `Node` instances in the order they appear in the document.
    ///
    /// # Example
    /// ```rust
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    ///
    /// let xml_data = b"<root><child>Text</child><last/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let all_nodes: Vec<Node> = document.all_nodes().collect();
    ///
    /// assert_eq!(all_nodes.len(), 4); // root, child, Text, totototo
    #[inline]
    #[must_use]
    pub fn all_nodes(&self) -> Nodes<'_> {
        Nodes::new(self)
    }

    /// Returns an iterator over the descendants of a given node.
    ///
    /// This method provides an iterator that traverses all descendant nodes of the specified node index.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node whose descendants are to be iterated over.
    ///
    /// # Returns
    /// - `Nodes`: An iterator that yields `Node` instances representing the descendants of the specified node.
    ///
    /// # Example
    /// ```rust
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    ///
    /// let xml_data = b"<root><child>Text</child><last/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let descendants: Vec<Node> = document.descendants(root_node.idx()).collect();
    ///
    /// assert_eq!(descendants.len(), 3); // child, Text, and last
    /// assert!(descendants[0].is("child"));
    /// assert_eq!(descendants[1].text().unwrap(), "Text");
    /// assert!(descendants[2].is("last"));
    /// ```
    #[inline]
    #[must_use]
    pub fn descendants(&self, node_idx: NodeIdx) -> Nodes<'_> {
        Nodes::descendants(self, node_idx)
    }

    /// Returns the last descendant of a given node index.
    ///
    /// This method finds the last descendant node of the specified node index in the document.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node whose last descendant is to be found.
    ///
    /// # Returns
    /// - `NodeIdx`: The index of the last descendant node.
    /// - `0`: If the node index is invalid or if there are no descendants for the root node.
    ///
    /// # Example
    /// ```rust
    /// use xhtml_parser::Document;
    /// use xhtml_parser::Node;
    ///
    /// let xml_data = b"<root><child>Text</child>boo<last/></root>".to_vec();
    /// let document = Document::new(xml_data).unwrap();
    /// let root_node = document.root().unwrap();
    /// let last_descendant_idx = document.last_descendant(root_node.idx());
    ///
    /// assert!(last_descendant_idx.is_some()); // There should be descendants
    /// let last_descendant = document.get_node(last_descendant_idx.unwrap()).unwrap();
    /// assert!(last_descendant.is("last")); // The last descendant should be "last"
    /// assert_eq!(document.last_descendant(last_descendant.idx()), None);
    /// ```
    ///
    /// # Notes
    /// - The method checks if the node index is valid and returns `0` if it is not.
    /// - If the node index is `0` or if it is the root node with no descendants, it returns `0`.
    /// # Errors
    /// - If the node index is invalid or out of bounds, it returns `0`.
    /// - If the node index is `1` and there are no descendants, it returns `0`.
    #[must_use]
    pub fn last_descendant(&self, node_idx: NodeIdx) -> Option<NodeIdx> {
        if node_idx == 0
            || self.nodes[node_idx as usize].first_child_idx() == 0
            || node_idx as usize >= (self.nodes.len() - 1)
        {
            None // Invalid node index, or there is no node following that node
        } else if node_idx == 1 {
            // If the node is the root, return the last node index
            Some(self.last_node_idx())
        } else {
            #[cfg(not(feature = "forward_only"))]
            {
                let mut up_idx = self.nodes[node_idx as usize].parent_idx;
                let mut last_descendant = self.nodes[up_idx as usize].next_sibling_idx();
                while last_descendant == 0 {
                    up_idx = self.nodes[up_idx as usize].parent_idx;
                    if up_idx <= 1 {
                        last_descendant = self.nodes.len() as NodeIdx; // No more parents, will return the last node_idx
                        break;
                    }
                    last_descendant = self.nodes[up_idx as usize].next_sibling_idx();
                }

                Some(last_descendant - 1)
            }

            #[cfg(feature = "forward_only")]
            {
                let mut curr_node_idx = self.nodes[node_idx as usize].first_child_idx();
                // Start from the first child of the node

                loop {
                    while self.nodes[curr_node_idx as usize].next_sibling_idx() != 0 {
                        curr_node_idx = self.nodes[curr_node_idx as usize].next_sibling_idx();
                    }
                    if self.nodes[curr_node_idx as usize].first_child_idx() != 0 {
                        curr_node_idx = self.nodes[curr_node_idx as usize].first_child_idx();
                    } else {
                        break; // Found the last descendant
                    }
                }
                Some(curr_node_idx)
            }
        }
    }

    /// Returns the next sequential node after the node index parameter.
    #[inline]
    #[must_use]
    pub fn next_seq_node(&self, current: NodeIdx) -> Option<Node<'_>> {
        let next = current + 1;
        if next < self.nodes.len() as NodeIdx {
            self.get_node(next).ok()
        } else {
            None
        }
    }

    /// Returns the previous sequential node before the node index parameter.
    #[inline]
    #[must_use]
    pub fn previous_seq_node(&self, current: NodeIdx) -> Option<Node<'_>> {
        let previous = current - 1;
        if previous > 0 {
            self.get_node(previous).ok()
        } else {
            None
        }
    }
}

impl fmt::Debug for Document {
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
                node: &Node,
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
                parent: &Node,
                indent: usize,
                f: &mut fmt::Formatter,
            ) -> Result<(), fmt::Error> {
                for child in parent.children() {
                    print_node(&child, indent, f)?;
                }

                Ok(())
            }

            writeln!(f, "Document [")?;
            print_node(&root, 1, f)?;
            writeln!(f, "]")?;

            Ok(())
        } else {
            write!(f, "Document [No root node]")?;
            Ok(())
        }
    }
}

/// Iterator over nodes.
///
/// This iterator traverses nodes in the document in the sequence of appearance in the XML content.
/// It starts from the first node and goes through all of them in the order of the vector that contains the nodes.
/// It is used for both iterating all nodes of a document or all descendants of a specific node.
///
/// # Example
/// ```rust
/// use xhtml_parser::Document;
///
/// let xml_data = b"<root><child>Text</child><totototo/></root>".to_vec();
/// let document = Document::new(xml_data).unwrap();
/// let all_nodes = document.all_nodes().collect::<Vec<_>>();
///
/// assert_eq!(all_nodes.len(), 4); // root, child, Text, totototo
/// assert!(all_nodes[0].is("root"));
/// assert!(all_nodes[1].is("child"));
/// assert_eq!(all_nodes[2].text().unwrap(), "Text");
/// assert!(all_nodes[3].is("totototo"));
/// assert!(all_nodes[3].is_element()); // totototo is an element node
/// assert!(all_nodes[2].is_text()); // Text is a text node
/// assert!(all_nodes[0].is_root()); // root is the root node
/// assert!(all_nodes[1].is_element()); // child is an element node
/// assert!(all_nodes[0].has_children()); // root has children
/// assert!(all_nodes[1].has_children()); // child has children
/// assert!(!all_nodes[2].has_children()); // Text does not have children
/// assert!(!all_nodes[3].has_children()); // totototo does not have children
/// ```
pub struct Nodes<'a> {
    front: Option<Node<'a>>,
    back: Option<Node<'a>>,
}

impl<'a> Nodes<'a> {
    /// Creates a new `Nodes` iterator for the given document.
    ///
    /// # Arguments
    /// - `document`: The document whose nodes will be iterated over.
    ///
    /// # Returns
    /// - `Nodes`: An iterator that yields `Node` instances representing the nodes in the document.
    #[inline]
    #[must_use]
    pub fn new(document: &'a Document) -> Self {
        let last_node_idx = document.last_node_idx();
        if last_node_idx == 0 {
            return Nodes {
                front: None,
                back: None,
            };
        }

        let front = document.get_node(1);
        let back = document.get_node(last_node_idx);
        Nodes {
            front: front.ok(),
            back: back.ok(),
        }
    }

    /// Creates a new `Nodes` iterator for the descendants of a given node index.
    ///
    /// # Arguments
    /// - `document`: The document whose descendants will be iterated over.
    /// - `node_idx`: The index of the node whose descendants are to be iterated over.
    ///
    /// # Returns
    /// - `Nodes`: An iterator that yields `Node` instances representing the descendants of the specified node.
    #[inline]
    #[must_use]
    pub fn descendants(document: &'a Document, node_idx: NodeIdx) -> Self {
        match document.last_descendant(node_idx) {
            None => Nodes {
                front: None,
                back: None,
            },
            Some(last_node_idx) => Nodes {
                front: document.get_node(node_idx + 1).ok(),
                back: document.get_node(last_node_idx).ok(),
            },
        }
    }
}

impl<'a> Iterator for Nodes<'a> {
    type Item = Node<'a>;

    /// Returns the next node in the sequence.
    ///
    /// This method retrieves the next node in the sequence of nodes in the document.
    ///
    /// # Returns
    /// - `Some(Node)`: The next node in the sequence.
    /// - `None`: If there are no more nodes to iterate over.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            let node = self.front.take();
            self.back = None;
            node
        } else {
            let node = self.front.take();
            self.front = node.as_ref().and_then(|n| n.doc.next_seq_node(n.idx()));
            node
        }
    }
}

#[cfg(not(feature = "forward_only"))]
impl DoubleEndedIterator for Nodes<'_> {
    /// Returns the previous node in the sequence.
    ///
    /// This method retrieves the previous node in the sequence of nodes in the document.
    ///
    /// # Returns
    /// - `Some(Node)`: The previous node in the sequence.
    /// - `None`: If there are no more nodes to iterate over in the reverse direction.
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == self.front {
            let node = self.back.take();
            self.front = None;
            node
        } else {
            let node = self.back.take();
            self.back = node.as_ref().and_then(|n| n.doc.previous_seq_node(n.idx()));
            node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let xml_data = b"<root><child>Text</child><totototo/></root>".to_vec();
        let document = Document::new(xml_data).unwrap();

        println!("Document created: {:#?}", document);
    }
}
