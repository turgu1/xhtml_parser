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
//! - `default`: Enables the default features of the parser.
//! - `namespace_removal`: Enables removal of XML namespaces from tag names during parsing. Default is **enabled**.
//! - `parse_escapes`: Enables parsing of character escapes sequences (`&..;`) in PCData nodes and attribute values. Default is **enabled**.
//! - `keep_ws_only_pcdata`: all PCData nodes that are composed of whitespace only will be kept. Default is **disabled**.
//! - `trim_pcdata`: trim whitespaces at beginning and end of PCData nodes. Default is **disabled**.
//!
//! # ChangeLog
//!
//! ## [0.2.1] - 2025-06-15
//!
//! - Date adjustment in changelog
//! - Added the changelog to README.md
//!
//! ## [0.2.0] - 2025-06-15
//!
//! - Going to version [0.2.0] is required as the way that space characters present at the beginning and end of PCData nodes are processed is different, whether or not the following added features are enabled or disabled.
//! - Added `keep_ws_only_pcdata`: all PCData nodes that are composed of whitespace only will be kept. Default is **disabled**.
//! - Added `trim_pcdata`: trim whitespaces at beginning and end of PCData nodes. Default is **disabled**.
//! - Corrected the description of the `parse_escapes` feature to add `attribute values` that are parsed for escapes sequences when that feature is enabled.
//! - Added test case for each feature. Requires to adjust selected feature before launching the individual tests.
//!
//! ## [0.1.2] - 2025-06-12
//!
//! - The Document `parser` method is no longer public outside of this crate.
//! - Added `Nodes` iterator to access document nodes in the sequence of creation. Accessible through the `Document::all_nodes()`, `Document::descendants()` and `Node::descendant()` methods.
//! - Added blank lines in the doc examples for better readability.
//! - Adjusted all examples to diminish the required `use` declarations.
//!
//! ## [0.1.1] - 2025-06-11
//!
//! - Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules
//! - Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
//! - Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.
//!
//! - Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.
//!
//! ## [0.1.0] - 2025-06-10
//!
//! Initial release
//!
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
