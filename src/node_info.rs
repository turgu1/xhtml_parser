/// Represents information about a node in the XML/HTML tree structure.
///
/// `NodeInfo` contains references to the node's parent, siblings, and children,
/// as well as its type.
///
/// # Fields
/// - `parent_idx`: The index of the parent node. `0` indicates the root node.
/// - `prev_sibling`: The index of the previous sibling node, or the last child of the parent if this is the first child.
/// - `next_sibling`: The index of the next sibling node, or the node following the parent.
/// - `first_child`: The index of the first child node of this node.
/// - `node_type`: The type of this node (e.g., element, text, comment).
use crate::defs::{NodeIdx, XmlIdx};
use crate::node_type::NodeType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    //node_idx: NodeIdx,   // Could never be 0, as 0 is reserved for None
    pub(crate) parent_idx: NodeIdx, // Parent node index, 0 for root
    prev_sibling: NodeIdx,          // previous sibling, or last child of parent
    next_sibling: NodeIdx,          // Could be next_sibling or the node following the parent
    first_child: NodeIdx,           // First child of this node
    node_type: NodeType,
}

impl<'xml> NodeInfo {
    /// Creates a new `NodeInfo` instance.
    ///
    /// # Arguments
    /// - `node_idx`: The index of the node (not used in this struct, but could be useful for other purposes).
    /// - `parent_idx`: The index of the parent node.
    /// - `node_type`: The type of the node (e.g., element, text, comment).
    #[inline]
    pub fn new(node_idx: NodeIdx, parent_idx: NodeIdx, node_type: NodeType) -> Self {
        NodeInfo {
            //node_idx,
            parent_idx,
            next_sibling: 0,
            prev_sibling: node_idx, // Initially set to itself
            first_child: 0,
            node_type,
        }
    }

    /// Returns `true` if this node is an element node.
    #[inline]
    pub fn is_element(&self) -> bool {
        matches!(self.node_type, NodeType::Element { .. })
    }

    /// Returns the index of the parent node, or `None` if this is the head node.
    #[inline]
    pub fn parent_idx(&self) -> Option<NodeIdx> {
        if self.parent_idx == 0 {
            None // Root node has no parent
        } else {
            Some(self.parent_idx)
        }
    }

    /// Returns the index of the previous sibling of this node.
    #[inline]
    pub fn prev_sibling_idx(&self) -> NodeIdx {
        self.prev_sibling
    }

    /// Returns the index of the next sibling of this node.
    #[inline]
    pub fn next_sibling_idx(&self) -> NodeIdx {
        self.next_sibling
    }

    /// Returns the index of the first child of this node.
    #[inline]
    pub fn first_child_idx(&self) -> NodeIdx {
        self.first_child
    }

    /// Returns the position of this node in the XML source.
    ///
    /// For Element nodes, this is the start position of the element name.
    /// For Text nodes, this is the start position of the text content.
    /// For the head node, this is always `0`.
    #[inline]
    pub fn position(&self) -> XmlIdx {
        match &self.node_type {
            NodeType::Element { name, .. } => name.start,
            NodeType::Text(range) => range.start,
            NodeType::Head => 0,
        }
    }

    /// Returns the type of this node.
    #[inline]
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// Sets the next sibling index for this node.
    #[inline]
    pub fn set_next_sibling_idx(&mut self, idx: NodeIdx) {
        self.next_sibling = idx;
    }

    /// Sets the previous sibling index for this node.
    #[inline]
    pub fn set_prev_sibling_idx(&mut self, idx: NodeIdx) {
        self.prev_sibling = idx;
    }

    /// Sets the first child index for this node.
    #[inline]
    pub fn set_first_child_idx(&mut self, idx: NodeIdx) {
        self.first_child = idx;
    }

    /// Sets the parent index for this node.
    #[inline]
    pub fn set_parent_idx(&mut self, idx: NodeIdx) {
        self.parent_idx = idx;
    }

    // set the node NoteType
    #[inline]
    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }
}
