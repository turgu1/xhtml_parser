## Rust XHTML Tree Parser

This is a simple XML/XHTML parser that constructs a read-only tree structure similar to a DOM from an `Vec<u8>` XML/XHTML file representation. This is used by the author for EPub reader embedded applications.

Loosely based on the PUGIXML parsing method and structure that is described [here](https://aosabook.org/en/posa/parsing-xml-at-the-speed-of-light.html), it is an in-place parser: all strings are kept in the received `Vec<u8>` for which the parser takes ownership. Its content is modified to expand entities to their UTF-8 representation (in attribute values and PCData). Position index of elements is preseved in the vector. Tree nodes are kept to their minimum size for low-memory-constrained environments. A single pre-allocated vector contains all the nodes of the tree. 

The parsing process is limited to normal tags, attributes, and PCData content. No processing instruction (`<? .. ?>`), comment (`<!-- .. -->`), CDATA (`<![CDATA .. ]>`), DOCTYPE (`<!DOCTYPE .. >`), or DTD inside DOCTYPE (`[ ... ]`) is retrieved. Basic validation is done to the XHTML structure to ensure content coherence.

- No `unsafe` construct.
- XML content must be UTF-8.
- Namespace prefix are removed from tag and attribute names (`namespace_removal` feature).
- Standard XML entities (`&amp;`, `&lt;`, `&gt;`, `&apos;`, and `&quot;`), Unicode numerical character references (`&#xhhhh;` and `&#nnnn;`), and XHTML-related entities (as described [here](https://www.w3.org/TR/xhtml-modularization/dtd_module_defs.html#a_dtd_xhtml_character_entities)) are translated to their UTF-8 representation (`parse_escapes` feature).

The parser is open-source and can be freely used and modified under the terms of the MIT license.

### Cargo defined Features
- `default`: Enables the default features of the parser. 
- `namespace_removal`: Enables removal of XML namespaces from tag names during parsing. Default is **enabled**.
- `parse_escapes`: Enables parsing of character escapes sequences (`&..;`) in PCData nodes and attribute values. Default is **enabled**.
- `keep_ws_only_pcdata`: all PCData nodes that are composed of whitespace only will be kept. Default is **disabled**.
- `trim_pcdata`: trim whitespaces at beginning and end of PCData nodes. Default is **disabled**.

# ChangeLog

## [0.2.3] - TBA

- Attribute value normalization.
- removal of carriage_return characters in attribute values and PCData.

## [0.2.2] - 2025-06-17

- Better Comment, `<!DOCTYPE .. >` and `<![CDATA[ .. ]]>` bypassing parser algorithm. 
- Added DTD bypassing.
- Added tests for these.
- Corrected README.md.

## [0.2.1] - 2025-06-15

- Date adjustment in changelog.
- Added the changelog to README.md.

## [0.2.0] - 2025-06-15

- Going to version [0.2.0] is required as the way that space characters present at the beginning and end of PCData nodes are processed is different, whether or not the following added features are enabled or disabled.
- Added `keep_ws_only_pcdata`: all PCData nodes that are composed of whitespace only will be kept. Default is **disabled**.
- Added `trim_ws_pcdata`: trim whitespaces at beginning and end of PCData nodes. Default is **disabled**.
- Corrected the description of the `parse_escapes` feature to add `attribute values` that are parsed for escapes sequences when that feature is enabled.
- Added test case for each feature. Requires to adjust selected feature before launching the individual tests.

## [0.1.2] - 2025-06-12

- The Document `parser` method is no longer public outside of this crate.
- Added `Nodes` iterator to access document nodes in the sequence of creation. Accessible through the `Document::all_nodes()`, `Document::descendants()` and `Node::descendants()` methods.
- Added blank lines in the doc examples for better readability.
- Adjusted all examples to diminish the required `use` declarations.

## [0.1.1] - 2025-06-11

- Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules.
- Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
- Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.

- Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.

## [0.1.0] - 2025-06-10 

Initial release.

