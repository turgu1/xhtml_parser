//! This is the `xhtml-parser` crate, which provides an XML/XHTML parser in Rust.
//!
//! It implements a simple XML parser that reads XML content, identifies tags, attributes, and text content,
//! and constructs a document structure from the parsed data. The parser handles various XML constructs,
//! including elements, attributes, text nodes, comments, and processing instructions.
//!
//! The parser is designed to be efficient and robust, with error handling for malformed XML.
//! The code is structured to allow for easy extension and modification, with clear separation of concerns.
//! The parser uses a state machine to manage the parsing process, transitioning between different states.
//! It includes functionality for handling character entities and whitespace normalization.
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
