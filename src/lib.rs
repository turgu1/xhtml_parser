//! This is the `xhtml-parser` crate, which provides an XML/XHTML parser in Rust.
//!
//! It implements a simple XML parser that reads XML content, identifies tags, attributes, and text content,
//! and constructs a document structure from the parsed data. The parser handles various XML constructs,
//! including elements, attributes, text nodes (`PCData`).
//!
//! Loosely based on the PUGIXML parsing method and structure that is described
//! [here](https://aosabook.org/en/posa/parsing-xml-at-the-speed-of-light.html), it is an in-place parser:
//! modified to expand entities to their UTF-8 representation (in attribute values and `PCData`). Position index of
//! elements is preserved in the vector. Tree nodes are kept to their minimum size for low-memory-constrained
//! environments. A single pre-allocated vector contains all the nodes of the tree.
//!
//! The parser is designed to be efficient and robust, with error handling for malformed XML.
//! The code is structured to allow for easy extension and modification, with clear separation of concerns.
//! The parser uses a state machine to manage the parsing process, transitioning between different states.
//! It includes functionality for handling character entities and whitespace normalization.
//!
//! The code is organized into modules, with a focus on clarity and maintainability.
//! It is capable of handling a wide range of XML/XHTML documents, including those with namespaces and complex structures.
//!
//! Node and Attribute vector index sizes, as well as the maximum XML file size, are configurable via features. The associated features permit you to adjust the size of structs required for the DOM tree to optimize memory usage.
//!
//! For vaious examples of usage, please refer to the documentation and tests provided in the repository.
//!
//! ## Cargo defined Features
//!
//! - `default`: Enables the default features of the parser.
//! - `namespace_removal`: Enables removal of XML namespaces from tag names during parsing. Default is **enabled**.
//! - `parse_escapes`: Enables parsing of character escapes sequences (`&..;`) in `PCData` nodes. Default is **enabled**.
//! - `keep_ws_only_pcdata`: all `PCData` nodes that are composed of whitespace only will be kept. Default is *disabled*.
//! - `trim_pcdata`: trim whitespaces at beginning and end of `PCData` nodes. Default is *disabled*.
//! - `small_node_count`: Uses 16-bit indices for the nodes vector. Default is **enabled**.
//! - `medium_node_count`: Uses 32-bit indices for the nodes vector. Default is *disabled*.
//! - `large_node_count`: Uses 64-bit indices for the nodes vector. Default is *disabled*.
//! - `small_attr_count`: Uses 16-bit indices for the attributes vector. Default is **enabled**.
//! - `medium_attr_count`: Uses 32-bit indices for the attributes vector. Default is *disabled*.
//! - `large_attr_count`: Uses 64-bit indices for the attributes vector. Default is *disabled*.
//! - `small_xml_size`: Allow XML files up to 64KB in length. Default is *disabled*.
//! - `medium_xml_size`: Allow XML files up to 4GB in length. Default is **enabled**.
//! - `large_xml_size`: Allow XML files up to 16 Hexa-Bytes in length. Default is *disabled*.
//! - `use_cstr`: Uses an index into a null-terminated `[u8]` slice (C-style string) instead of a `Range` to represent string locations in the XML content. Default is *disabled*.
//! - `forward_only`: Removes node information and methods that permit going backward in the node structure. Default is *disabled*.
//! - `all_features` to get all features enabled under a single one, but without the following: `xxxx_node_count`, `xxxx_attr_count`, and `xxxx_xml_size`.
//!
//! ## Basic performance comparison
//!
//! For performance comparison, a series of 20 runs were done with both PUGIXML (GNU C++) and this crate, using `-O3` optimization and parsing the same 5.5 MB XML file containing 25K nodes and 25K attributes. Used the last version of PUGIXML and this crate with the default options. The values shown are the average summation of the durations with their standard deviation. Results may vary depending on the computer performance and many other aspects (system load, operating system, compiler versions, enabled options/features, data caching, etc.).
//!
//! |                  | `PUGIXML` | `XHTML_PARSER` |
//! |------------------|:-------:|:------------:|
//! | Average Duration | 5856 µS |   3380 µS    |
//! | Std Deviation    |  266 µS |     88 µS    |
//!
//! ## Effects of some features on node structure element size
//!
//! Here is a table showing the effect that some feature combinaisons may have on the DOM-like structure sizes. The nodes and attributes structure element sizes are shown (separated with a '/'), depending on the following features:
//!
//! - `none`, `use_cstr`, `forward_only`, `use_cstr` and `forward_only` combined.
//! - `xxxx_node_count`, `xxxx_attr_count`, `xxxx_xml_size`.
//!
//! <style scoped>
//! table {
//!   font-size: 12px;
//! }
//! </style>
//!
//! |                                                           |   `none`   | `use_cstr` | `forward_`</br>`only` | `use_cstr` &</br>`forward_only` |
//! |-----------------------------------------------------------|:----------:|:----------:|:--------------:|:---------------------------:|
//! | `small_node_count`</br>`small_attr_count`</br>`small_xml_size`    |   18 / 8   |   16 / 4   |     14 / 8     |             12 / 4          |
//! | `small_node_count`</br>`small_attr_count`</br>`medium_xml_size`   |   24 / 16  |   20 / 8   |     20 / 16    |             16 / 8          |
//! | `medium_node_count`</br>`medium_attr_count`</br>`medium_xml_size` |   36 / 16  |   32 / 8   |     28 / 16    |             24 / 8          |
//! | `medium_node_count`</br>`medium_attr_count`</br>`large_xml_size`  |   48 / 32  |   40 / 16  |     40 / 32    |             32 / 16         |
//!
//! ## Licensing
//!
//! The parser is open-source and can be freely used and modified under the terms of the MIT license.
//!
//! ## `ChangeLog`
//!
//! ### [0.2.10] - 2025-07-15
//!
//! - Added byte slice retrieval methods for node names, attribute names and values, and `PCData`.
//! - Some performance optimization for the `Document::check_closing_tag()` method located in the `parser.rs` file.
//! - Table display adjustment in README.md (no `style` tag allowed in github).
//!
//! ### [0.2.9] - 2025-07-14
//!
//! - Added 75 negative tests for potentially malformed XML content. Some method adjustments in support of malformed content processing.
//! - Added `CStr` retrieval methods for node names, attribute names and values, and `PCData` when the `use_cstr` feature is enabled.
//! - Tables formatting adjusted in documentation and readme file.
//!
//! ### [0.2.8] - 2025-07-11
//!
//! - New `forward_only` feature: This feature removes node information and methods that permit going backward in the node structure. This is to diminish the amount of memory required to keep the nodes structure, useful for memory-constrained context when backward displacement is not used. See the section on size effects for more information, combined or not with the `use_cstr` feature.
//!
//! - Some code refactoring.
//!
//! ### [0.2.7] - 2025-07-01
//!
//! - Clippy (and pedantic) related refactoring.
//! - `State::ReadTagClose` parsing update. `parser::translate_sequence()` method revisited.
//! - Methods' `#[inline]` adjustments for better performance.
//! - Adjusted performance results after testing.
//! - Added `small_attr_count`, `medium_attr_count`, and `large_attr_count` features to use 16, 32, or 64-bit indices for the attributes vector, respectively. `small_attr_count` is the default value.
//! - Added `small_xml_size`, `medium_xml_size`, and `large_xml_size` features to accept xml file with a maximum size of 64KB (16-bit indices), 4GB (32-bit indices), or 16 Hexa-Bytes (64-bit indices) respectively. `medium_xml_size` is the default value.
//! - The Document creation method now checks if the received XML vector is not too large for the selected capacity of nodes, attributes and file size.
//!
//! ### [0.2.6] - 2025-06-30
//!
//! Performance comparison revisited.
//! - Using `memchr` for char search instead of `.iter().position()`. Can easily be changed through the `parser::seach_char!()` macro.
//! - The `memchr` crate is used without the `std` option.
//! - Using the last PUGIXML version with default options to redo performance comparison.
//! - Performance table adjusted accordingly.
//!
//! ### [0.2.5] - 2025-06-29
//!
//! - Restrict visibility of some methods to the crate.
//! - Basic performance comparison with PUGIXML.
//! - Code refactoring.
//! - phf crate version is now 0.12 .
//!
//! ### [0.2.4] - 2025-06-29
//!
//! - New feature: `use_cstr`: By using indices into null-terminated `[u8]` slices instead of a range of indices (to keep the location of strings located in the XML document), this feature reduces the size of nodes to 20 bytes instead of 24 (17% gain in size for each node). For attributes, the size is reduced from 16 bytes to 8 bytes (50% gain in size for each attribute). This change optimizes the memory required to keep the XML DOM-like tree accessible, which is particularly beneficial for embedded applications where available memory is limited. Note that using this feature reduces the overall performance of the parser by approximately 5% to 10%.
//! - New `all_features` to get all features enabled under a single one, but without the following: `small_node_count`, `medium_node_count`, and `large_node_count`.
//! - A new test case for performance computation was added.
//! - The `no_feature` feature was removed.
//!
//! ### [0.2.3] - 2025-06-23
//!
//! - Attribute value normalization: Whitespace (space, tab, carriage-return, line-feed) at the beginning and end of attribute values are removed. All other whitespace character sequences are replaced with a single space. All known entities (`&..;`) are translated.
//! - In `PCData`, carriage-return characters alone are replaced with a line-feed character; carriage-return are removed when followed by a line-feed.
//! - All parser macros are replaced with their equivalent inline method. This to simplify debugging and for better readability.
//! - Correction: with the `keep_ws_only_pcdata` feature enabled, whitespace only nodes are created after a first element tag is encountered.
//! - The parsing process is now ending once the first element is completely parsed (its ending tag has been encountered). All remaining content in the XML file is ignored.
//! - `Chartype` enum cleanup.
//! - Added `no_feature` feature.
//! - Added `small_node_count`, `medium_node_count`, and `large_node_count` features to use 16, 32, or 64-bit indices for the nodes vector, respectively. `small_node_count` is the default value.
//!
//! ### [0.2.2] - 2025-06-17
//!
//! - Better Comment, `<!DOCTYPE .. >` and `<![CDATA[ .. ]]>` bypassing parser algorithm.
//! - Added DTD bypassing.
//! - Added tests for these.
//! - Corrected README.md.
//!
//! ### [0.2.1] - 2025-06-15
//!
//! - Date adjustment in changelog.
//! - Added the changelog to README.md.
//!
//! ### [0.2.0] - 2025-06-15
//!
//! - Going to version [0.2.0] is required as the way that space characters present at the beginning and end of `PCData` nodes are processed is different, whether or not the following added features are enabled or *disabled*.
//! - Added `keep_ws_only_pcdata`: all `PCData` nodes that are composed of whitespace only will be kept. Default is *disabled*.
//! - Added `trim_pcdata`: trim whitespaces at beginning and end of `PCData` nodes. Default is *disabled*.
//! - Corrected the description of the `parse_escapes` feature to add `attribute values` that are parsed for escapes sequences when that feature is enabled.
//! - Added test case for each feature. Requires to adjust selected feature before launching the individual tests.
//!
//! ### [0.1.2] - 2025-06-12
//!
//! - The Document `parser` method is no longer public outside of this crate.
//! - Added `Nodes` iterator to access document nodes in the sequence of creation. Accessible through the `Document::all_nodes()`, `Document::descendants()` and `Node::descendants()` methods.
//! - Added blank lines in the doc examples for better readability.
//! - Adjusted all examples to diminish the required `use` declarations.
//!
//! ### [0.1.1] - 2025-06-11
//!
//! - Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules.
//! - Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
//! - Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.
//!
//! - Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.
//!
//! ### [0.1.0] - 2025-06-10
//!
//! Initial release.
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
