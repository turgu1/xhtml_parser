## ChangeLog

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

