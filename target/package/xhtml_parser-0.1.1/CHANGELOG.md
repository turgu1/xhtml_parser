# ChangeLog

## [0.1.0] - 2025-06-10 

Initial release

## [0.1.1] - 2025-06-11

- Added `pub fn is(&self, name: &str) -> bool` method to `Attribute` and `Node` modules
- Added  `pub use` entries in `lib.rs` to simplify usage in calling applications. All examples and tests have been modified in accordance with this change.
- Added `Display` trait definition for the `ParseXmlError` enum in the `defs` module.

- Removed the `position` field of the `node_info` struct as the information is available through the range fields of the `NodeType` enum.
