# ChangeLog

## [0.1.2] - 2025-06-12

- The Document `parser` method is no longer public outside of this crate.
- Added `Nodes` iterator to access document nodes in the sequence of creation. Accessible through the `Document::all_nodes()`, `Document::descendants()` and `Node::descendant()` methods.
- Added blank lines in the doc examples for better readability.
- Adjusted all examples to diminish the required `use` declarations.

## [0.1.1] - 2025-06-11

- Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules
- Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
- Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.

- Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.

## [0.1.0] - 2025-06-10 

Initial release

