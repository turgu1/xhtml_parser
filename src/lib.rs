//! This is the `xhtml-parser` crate, which provides an XML/XHTML parser in Rust.
//!
//! It implements a simple XML parser that reads XML content, identifies tags, attributes, and text content,
//! and constructs a document structure from the parsed data. The parser handles various XML constructs,
//! including elements, attributes, text nodes (PCData).
//!
//! Loosely based on the PUGIXML parsing method and structure that is described
//! [here](https://aosabook.org/en/posa/parsing-xml-at-the-speed-of-light.html), it is an in-place parser:
//! modified to expand entities to their UTF-8 representation (in attribute values and PCData). Position index of
//! elements is preserved in the vector. Tree nodes are kept to their minimum size for low-memory-constrained
//! environments. A single pre-allocated vector contains all the nodes of the tree.
//!
//! The parser is designed to be efficient and robust, with error handling for malformed XML.
//! The code is structured to allow for easy extension and modification, with clear separation of concerns.
//! The parser uses a state machine to manage the parsing process, transitioning between different states.
//! It includes functionality for handling character entities and whitespace normalization.
//!
//!
//! The code is organized into modules, with a focus on clarity and maintainability.
//! It is capable of handling a wide range of XML/XHTML documents, including those with namespaces and complex structures.
//!
//! The parser is open-source and can be freely used and modified under the terms of the MIT license.
//!
//! For vaious examples of usage, please refer to the documentation and tests provided in the repository.
//!
//! # Cargo defined Features
//!
//! - `default`: Enables the default features of the parser. All of the following features are enabled by default:
//! - `namespace_removal`: Enables removal of XML namespaces from tag names during parsing.
//! - `parse_escapes`: Enables parsing of character escapes sequences (`&..;`) in text nodes.

pub mod attribute;
pub mod defs;
pub mod document;
pub mod node;
pub mod node_info;
pub mod node_type;
pub mod parser;

pub use attribute::Attribute;
pub use document::Document;
pub use node::Node;
pub use node_type::NodeType;
