## Rust XHTML Tree Parser

This is a simple XML/XHTML parser that constructs a read-only tree structure similar to a DOM from an `Vec<u8>` XML/XHTML file representation. This is used by the author for EPub reader embedded applications.

Loosely based on the PUGIXML parsing method and structure that is described [here](https://aosabook.org/en/posa/parsing-xml-at-the-speed-of-light.html), it is an in-place parser: all strings are kept in the received `Vec<u8>` for which the parser takes ownership. Its content is modified to expand entities to their UTF-8 representation (in attribute values and PCData). Position index of elements is preseved in the vector. Tree nodes are kept to their minimum size for low-memory-constrained environments. A single pre-allocated vector contains all the nodes of the tree. 

The parsing process is limited to normal tags, attributes, and PCData content. No processing instruction (`<? .. ?>`), comment (`<!-- .. -->`), CDATA (`<![CDATA .. ]>`), or DOCTYPE (`<!DOCTYPE .. ]>`) is retrieved. Basic validation is done to the XHTML structure to ensure content coherence.

- No `unsafe` construct.
- XML content must be UTF-8.
- Namespace prefix are removed from tag and attribute names.
- Standard XML entities (`&amp;`, `&lt;`, `&gt;`, `&apos;`, and `&quot;`), Unicode numerical character references (`&#xhhhh;` and `&#nnnn;`), and XHTML-related entities (as described [here](https://www.w3.org/TR/xhtml-modularization/dtd_module_defs.html#a_dtd_xhtml_character_entities)) are translated to their UTF-8 representation.

The parser is open-source and can be freely used and modified under the terms of the MIT license.

### Cargo defined Features
- `default`: Enables the default features of the parser. All of the following features are enabled by default:
- `namespace_removal`: Enables removal of XML namespaces from tag names during parsing.
- `parse_escapes`: Enables parsing of character escapes sequences (`&..;`) in text nodes.
- `verbose`: Enable information messages during parsing.
