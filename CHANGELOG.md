## ChangeLog

### [0.2.9] - 2025-07-14

- Added 75 negative tests for potentially malformed XML content. Some method adjustments in support of malformed content processing.
- Added CStr retrieval methods for node names, attribute names and values, and PCData when the `use_cstr` feature is enabled.
- Tables formatting adjusted in documentation and readme file.

### [0.2.8] - 2025-07-11

- New `forward_only` feature: This feature removes node information and methods that permit going backward in the node structure. This is to diminish the amount of memory required to keep the nodes structure, useful for memory-constrained context when backward displacement is not used. See the `README.md` file, section on size effects for more information, combined or not with the `use_cstr` feature.
- Some code refactoring.

### [0.2.7] - 2025-07-01

- Clippy (and pedantic) related refactoring.
- State::ReadTagClose parsing update. parser::translate_sequence() method revisited.
- Methods' `#[inline]` adjustments for better performance.
- Adjusted performance results after testing.
- Added `small_attr_count`, `medium_attr_count`, and `large_attr_count` features to use 16, 32, or 64-bit indices for the attributes vector, respectively. `small_attr_count` is the default value.
- Added `small_xml_size`, `medium_xml_size`, and `large_xml_size` features to accept xml file with a maximum size of 64KB (16-bit indices), 4GB (32-bit indices), or 16 HexaBytes (64-bit indices) respectively. `medium_xml_size` is the default value.
- The Document creation method now checks if the received XML vector is not too large for the selected capacity of nodes, attributes and file size.

### [0.2.6] - 2025-06-30

Performance comparison revisited.
- Using `memchr` for char search instead of `.iter().position()`. Can easily be changed through the `parser::seach_char!()` macro.
- The `memchr` crate is used without the `std` option.
- Using the last PUGIXML version with default options to redo performance comparison. 
- Performance table adjusted accordingly.

### [0.2.5] - 2025-06-29

- Restrict visibility of some methods to the crate.
- Basic performance comparison with PUGIXML.
- Code refactoring.
- phf crate version is now 0.12 .

### [0.2.4] - 2025-06-29

- New feature: `use_cstr`: By using indices into null-terminated `[u8]` slices instead of a range of indices (to keep the location of strings located in the XML document), this feature reduces the size of nodes to 20 bytes instead of 24 (17% gain in size for each node). For attributes, the size is reduced from 16 bytes to 8 bytes (50% gain in size for each attribute). This change optimizes the memory required to keep the XML DOM-like tree accessible, which is particularly beneficial for embedded applications where available memory is limited. Note that using this feature reduces the overall performance of the parser by approximately 5% to 10%.
- New `all_features` to get all features enabled under a single one, but without the following: `small_node_count`, `medium_node_count`, and `large_node_count`.
- A new test case for performance computation was added.
- The `no_feature` feature was removed.

### [0.2.3] - 2025-06-23

- Attribute value normalization: Whitespace (space, tab, carriage-return, line-feed) at the beginning and end of attribute values are removed. All other whitespace character sequences are replaced with a single space. All known entities (`&..;`) are translated.
- In PCData, carriage-return characters alone are replaced with a line-feed character; carriage-return are removed when followed by a line-feed.
- All parser macros are replaced with their equivalent inline method. This to simplify debugging and for better readability.
- Correction: with the `keep_ws_only_pcdata` feature enabled, whitespace only nodes are created after a first element tag is encountered.
- The parsing process is now ending once the first element is completely parsed (its ending tag has been encountered). All remaining content in the XML file is ignored.
- `Chartype` enum cleanup.
- Added `no_feature` feature.
- Added `small_node_count`, `medium_node_count`, and `large_node_count` features to use 16, 32, or 64-bit indices for the nodes vector, respectively. `small_node_count` is the default value.

### [0.2.2] - 2025-06-17

- Better Comment, `<!DOCTYPE .. >` and `<![CDATA[ .. ]]>` bypassing parser algorithm. 
- Added DTD bypassing.
- Added tests for these.
- Corrected README.md.

### [0.2.1] - 2025-06-15

- Date adjustment in changelog.
- Added the changelog to README.md.

### [0.2.0] - 2025-06-15

- Going to version [0.2.0] is required as the way that space characters present at the beginning and end of PCData nodes are processed is different, whether or not the following added features are enabled or disabled.
- Added `keep_ws_only_pcdata`: all PCData nodes that are composed of whitespace only will be kept. Default is **disabled**.
- Added `trim_ws_pcdata`: trim whitespaces at beginning and end of PCData nodes. Default is **disabled**.
- Corrected the description of the `parse_escapes` feature to add `attribute values` that are parsed for escapes sequences when that feature is enabled.
- Added test case for each feature. Requires to adjust selected feature before launching the individual tests.

### [0.1.2] - 2025-06-12

- The Document `parser` method is no longer public outside of this crate.
- Added `Nodes` iterator to access document nodes in the sequence of creation. Accessible through the `Document::all_nodes()`, `Document::descendants()` and `Node::descendants()` methods.
- Added blank lines in the doc examples for better readability.
- Adjusted all examples to diminish the required `use` declarations.

### [0.1.1] - 2025-06-11

- Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules.
- Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
- Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.

- Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.

### [0.1.0] - 2025-06-10 

Initial release.

