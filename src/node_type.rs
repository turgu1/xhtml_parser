/// Represents the different types of nodes that can exist in an XML/XHTML document tree.
///
/// This enum defines the fundamental node types used in parsing and representing
/// XML/XHTML documents:
///
/// # Variants
///
/// * `Head` - The head node of the document tree, representing the document itself
/// * `Element` - An XML/XHTML element with a name and optional attributes
///   - `name`: Location information for the element's tag name in the source
///   - `attributes`: Range information for the element's attributes
/// * `Text` - A text node containing character data between elements
///   - Contains location information for the text content in the source
///
/// # Examples
///
/// ```rust
/// use crate::xhtml_parser::node_type::NodeType;
/// use crate::xhtml_parser::defs::{XmlLocation, AttributeRange};
///
/// // Head node
/// let head = NodeType::Head;
///
/// // Element node with name and attributes
/// let element = NodeType::Element {
///     name: 0..5,
///     attributes: 6..20,
/// };
///
/// // Text node
/// let text = NodeType::Text(25..35);
/// ```
use crate::defs::{AttributeRange, XmlLocation};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Eq)]
pub enum NodeType {
    Head,
    Element {
        name: XmlLocation,
        attributes: AttributeRange,
    },
    Text(XmlLocation),
}

/// Custom implementation of the `Debug` trait for `NodeType`.
///
/// This implementation provides human-readable debug output for the different
/// variants of `NodeType`:
///
/// - `Root`: Displays as "Root"
/// - `Element`: Displays as "Element(name: the_name_range, attributes: the_attributes_range)"
/// - `Text`: Displays as "Text(text_content_range)"
///
/// This is useful for debugging and logging purposes when working with the
/// node tree structure.
impl Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Head => write!(f, "Head"),
            NodeType::Element { name, attributes } => {
                write!(f, "Element(name: {:?}, attributes: {:?})", name, attributes)
            }
            NodeType::Text(text) => write!(f, "Text({:?})", text),
        }
    }
}
