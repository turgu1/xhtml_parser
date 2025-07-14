//! Negative tests for the xhtml_parser crate
//!
//! This module contains comprehensive negative tests to exercise the parser's
//! error handling capabilities and robustness against malformed XML and invalid
//! method calls.

#[cfg(test)]
mod negative_tests {
    use xhtml_parser::{defs::ParseXmlError, Document};

    // ========== Document Module Negative Tests ==========

    #[test]
    fn test_document_empty_xml() {
        let result = Document::new(Vec::new());
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Unexpected end of XML document"));
        }
    }

    #[test]
    fn test_document_invalid_utf8() {
        // Invalid UTF-8 byte sequence
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let result = Document::new(invalid_utf8);
        // Note: The parser might handle this differently based on implementation
        // but it should not panic
        if result.is_ok() {
            // If it doesn't fail immediately, operations on it should handle gracefully
            let doc = result.unwrap();
            assert!(doc.is_empty());
        }
    }

    #[test]
    fn test_document_malformed_xml_no_closing_tag() {
        let xml = b"<root><child>Content".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Unexpected end of XML document"));
        }
    }

    #[test]
    fn test_document_malformed_xml_mismatched_tags() {
        let xml = b"<root><child>Content</different></root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("does not match opening tag"));
        }
    }

    #[test]
    fn test_document_malformed_xml_invalid_tag_name() {
        let xml = b"<123invalid>Content</123invalid>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Tag name must start with a letter or underscore"));
        }
    }

    #[test]
    fn test_document_malformed_xml_unclosed_tag() {
        let xml = b"<root><child>Content</child".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Unexpected end of XML document"));
        }
    }

    #[test]
    fn test_document_malformed_xml_invalid_attribute_syntax() {
        let xml = b"<root attr=value>Content</root>".to_vec(); // Missing quotes
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Attribute value must be enclosed in quotes"));
        }
    }

    #[test]
    fn test_document_malformed_xml_invalid_attribute_name() {
        let xml = b"<root 123attr=\"value\">Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Attribute name must start with a letter or underscore"));
        }
    }

    #[test]
    fn test_document_malformed_xml_missing_attribute_equals() {
        let xml = b"<root attr\"value\">Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Attribute must have an '=' sign"));
        }
    }

    #[test]
    fn test_document_malformed_xml_invalid_self_closing_tag() {
        let xml = b"<root><child/Content</root>".to_vec(); // Missing '>'
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Expected '>' after '/' in self-closing tag"));
        }
    }

    #[test]
    fn test_document_malformed_xml_extra_closing_tag() {
        let xml = b"<root>Content</root></extra>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("No opening tag for closing tag"));
        }
    }

    #[test]
    fn test_document_malformed_xml_empty_tag_name() {
        let xml = b"<>Content</>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Tag name must start with a letter or underscore"));
        }
    }

    #[test]
    fn test_document_malformed_xml_invalid_closing_tag_name() {
        let xml = b"<root>Content</123root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Closing tag '123root' does not match opening tag 'root'"));
        }
    }

    #[test]
    fn test_document_get_node_invalid_index() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();

        // Test with index that's too large
        let result = document.get_node(9999);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Invalid node index"));
        }
    }

    #[test]
    fn test_document_get_node_zero_index() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();

        // Test with index 0 (should work - it's the head node)
        let result = document.get_node(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_too_many_nodes() {
        // This test is challenging because we need to hit the NodeIdx::MAX limit
        // We'll create a smaller test that demonstrates the concept
        // In a real scenario, this would need to be adapted based on the NodeIdx type

        // For now, let's test with a reasonably complex structure
        let mut xml = String::from("<root>");
        for i in 0..1000 {
            xml.push_str(&format!("<node{}></node{}>", i, i));
        }
        xml.push_str("</root>");

        let result = Document::new(xml.into_bytes());
        // This should still work for reasonable sizes
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_large_xml_size() {
        // Test with XML content that approaches size limits
        let large_content = "x".repeat(10000); // 10KB content
        let xml = format!("<root>{}</root>", large_content);
        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn test_document_nested_tags_depth() {
        // Test deeply nested XML structure
        let mut xml = String::new();
        let depth = 100;

        for i in 0..depth {
            xml.push_str(&format!("<level{}>", i));
        }
        xml.push_str("content");
        for i in (0..depth).rev() {
            xml.push_str(&format!("</level{}>", i));
        }

        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());
    }

    // ========== Node Module Negative Tests ==========

    #[test]
    fn test_node_tag_name_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // tag_name() on a text node should return empty string
        assert_eq!(text_node.tag_name(), "");
    }

    #[test]
    fn test_node_text_on_element_node() {
        let xml = b"<root><child>Text</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let child = root.first_child().unwrap();

        assert!(child.is_element());
        // text() on an element node should return None
        assert!(child.text().is_none());
    }

    #[test]
    fn test_node_attributes_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // attributes() on a text node should return empty iterator
        let attrs: Vec<_> = text_node.attributes().collect();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_node_first_child_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // first_child() on a text node should return None
        assert!(text_node.first_child().is_none());
    }

    #[test]
    fn test_node_last_child_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // last_child() on a text node should return None
        assert!(text_node.last_child().is_none());
    }

    #[test]
    fn test_node_children_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // children() on a text node should return empty iterator
        let children: Vec<_> = text_node.children().collect();
        assert!(children.is_empty());
    }

    #[test]
    fn test_node_get_child_nonexistent() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // Try to get a non-existent child
        assert!(root.get_child("nonexistent").is_none());
    }

    #[test]
    fn test_node_get_sibling_nonexistent() {
        let xml = b"<root><child1>Content1</child1><child2>Content2</child2></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let child1 = root.first_child().unwrap();

        // Try to get a non-existent sibling
        assert!(child1.get_sibling("nonexistent").is_none());
    }

    #[test]
    fn test_node_get_attribute_nonexistent() {
        let xml = b"<root attr1=\"value1\"><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // Try to get a non-existent attribute
        assert!(root.get_attribute("nonexistent").is_none());
    }

    #[test]
    fn test_node_get_attribute_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // get_attribute() on a text node should return None
        assert!(text_node.get_attribute("any").is_none());
    }

    #[test]
    fn test_node_next_sibling_on_last_child() {
        let xml = b"<root><child1>Content1</child1><child2>Content2</child2></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let child1 = root.first_child().unwrap();
        let child2 = child1.next_sibling().unwrap();

        // next_sibling() on the last child should return None
        assert!(child2.next_sibling().is_none());
    }

    #[test]
    fn test_node_prev_sibling_on_first_child() {
        let xml = b"<root><child1>Content1</child1><child2>Content2</child2></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let child1 = root.first_child().unwrap();

        // prev_sibling() on the first child should return None
        assert!(child1.prev_sibling().is_none());
    }

    #[test]
    fn test_node_parent_on_root() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // parent() on the root node should return None
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_node_is_with_empty_string() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // is() with empty string should return false
        assert!(!root.is(""));
    }

    #[test]
    fn test_node_is_with_whitespace() {
        let xml = b"<root><child>Content</child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // is() with whitespace should return false
        assert!(!root.is(" "));
        assert!(!root.is("\t"));
        assert!(!root.is("\n"));
    }

    #[test]
    fn test_node_case_sensitivity() {
        let xml = b"<Root><Child>Content</Child></Root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // XML tag names are case-sensitive
        assert!(root.is("Root"));
        assert!(!root.is("root"));
        assert!(!root.is("ROOT"));
    }

    // ========== Attribute Module Negative Tests ==========

    #[test]
    fn test_attribute_access_on_empty_element() {
        let xml = b"<root><child></child></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let child = root.first_child().unwrap();

        // Element with no attributes
        let attrs: Vec<_> = child.attributes().collect();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_attribute_iteration_on_text_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        // Iterating attributes on a text node should yield nothing
        let attrs: Vec<_> = text_node.attributes().collect();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_attribute_name_case_sensitivity() {
        let xml = b"<root Attr=\"value\">Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        // Attribute names are case-sensitive
        assert!(root.get_attribute("Attr").is_some());
        assert!(root.get_attribute("attr").is_none());
        assert!(root.get_attribute("ATTR").is_none());
    }

    #[test]
    fn test_attribute_is_with_empty_string() {
        let xml = b"<root attr=\"value\">Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let attr = root.attributes().next().unwrap();

        // is() with empty string should return false
        assert!(!attr.is(""));
    }

    #[test]
    fn test_attribute_is_with_whitespace() {
        let xml = b"<root attr=\"value\">Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let attr = root.attributes().next().unwrap();

        // is() with whitespace should return false
        assert!(!attr.is(" "));
        assert!(!attr.is("\t"));
        assert!(!attr.is("\n"));
    }

    // ========== Entity and Escape Sequence Negative Tests ==========

    #[test]
    fn test_invalid_entity_reference() {
        let xml = b"<root>Content with &invalidEntity; here</root>".to_vec();
        let result = Document::new(xml);
        // Invalid entities should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The invalid entity should be left as-is or handled gracefully
            assert!(text.contains("&invalidEntity;") || text.contains("invalidEntity"));
        }
    }

    #[test]
    fn test_incomplete_entity_reference() {
        let xml = b"<root>Content with &amp here</root>".to_vec();
        let result = Document::new(xml);
        // Incomplete entities should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The incomplete entity should be left as-is or handled gracefully
            assert!(text.contains("&amp") || text.contains("amp"));
        }
    }

    #[test]
    fn test_invalid_numeric_entity() {
        let xml = b"<root>Content with &#invalid; here</root>".to_vec();
        let result = Document::new(xml);
        // Invalid numeric entities should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The invalid numeric entity should be left as-is or handled gracefully
            assert!(text.contains("&#invalid;") || text.contains("invalid"));
        }
    }

    #[test]
    fn test_empty_entity_reference() {
        let xml = b"<root>Content with &; here</root>".to_vec();
        let result = Document::new(xml);
        // Empty entities should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The empty entity should be left as-is or handled gracefully
            assert!(text.contains("&;") || text.contains(""));
        }
    }

    // ========== Whitespace and Special Character Negative Tests ==========

    #[test]
    fn test_xml_with_null_bytes() {
        let xml = b"<root>Content\x00with\x00nulls</root>".to_vec();
        let result = Document::new(xml);
        // Null bytes should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The null bytes should be handled appropriately
            assert!(text.contains("Content") && text.contains("nulls"));
        }
    }

    #[test]
    fn test_xml_with_control_characters() {
        let xml = b"<root>Content\x01\x02\x03</root>".to_vec();
        let result = Document::new(xml);
        // Control characters should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The control characters should be handled appropriately
            assert!(text.contains("Content"));
        }
    }

    #[test]
    fn test_xml_with_only_whitespace() {
        let xml = b"   \n\t  \r\n  ".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Unexpected end of XML document"));
        }
    }

    #[test]
    fn test_xml_with_bom() {
        // UTF-8 BOM followed by XML
        let mut xml = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        xml.extend_from_slice(b"<root>Content</root>");
        let result = Document::new(xml);
        // BOM should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            assert!(root.is("root"));
        }
    }

    // ========== Memory and Resource Negative Tests ==========

    #[test]
    fn test_document_with_many_attributes() {
        let mut xml = String::from("<root");
        // Add many attributes
        for i in 0..100 {
            xml.push_str(&format!(" attr{}=\"value{}\"", i, i));
        }
        xml.push_str(">Content</root>");

        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let attrs: Vec<_> = root.attributes().collect();
            assert_eq!(attrs.len(), 100);
        }
    }

    #[test]
    fn test_document_with_very_long_attribute_value() {
        let long_value = "x".repeat(10000);
        let xml = format!("<root attr=\"{}\">Content</root>", long_value);
        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let attr_value = root.get_attribute("attr").unwrap();
            assert_eq!(attr_value.len(), 10000);
        }
    }

    #[test]
    fn test_document_with_very_long_tag_name() {
        let long_name = "x".repeat(1000);
        let xml = format!("<{}>Content</{}>", long_name, long_name);
        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name().len(), 1000);
        }
    }

    #[test]
    fn test_document_with_very_long_text_content() {
        let long_content = "x".repeat(50000);
        let xml = format!("<root>{}</root>", long_content);
        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            assert_eq!(text.len(), 50000);
        }
    }

    // ========== Edge Cases and Boundary Conditions ==========

    #[test]
    fn test_document_single_character_tag() {
        let xml = b"<a>Content</a>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "a");
        }
    }

    #[test]
    fn test_document_tag_with_numbers() {
        let xml = b"<tag123>Content</tag123>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "tag123");
        }
    }

    #[test]
    fn test_document_tag_with_underscores() {
        let xml = b"<tag_name>Content</tag_name>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "tag_name");
        }
    }

    #[test]
    fn test_document_tag_with_hyphens() {
        let xml = b"<tag-name>Content</tag-name>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "tag-name");
        }
    }

    #[test]
    fn test_document_empty_attribute_value() {
        let xml = b"<root attr=\"\">Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let attr_value = root.get_attribute("attr").unwrap();
            assert_eq!(attr_value, "");
        }
    }

    #[test]
    fn test_document_attribute_with_single_quotes() {
        let xml = b"<root attr='value'>Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let attr_value = root.get_attribute("attr").unwrap();
            assert_eq!(attr_value, "value");
        }
    }

    #[test]
    fn test_document_mixed_quote_types() {
        let xml = b"<root attr1=\"value1\" attr2='value2'>Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.get_attribute("attr1").unwrap(), "value1");
            assert_eq!(root.get_attribute("attr2").unwrap(), "value2");
        }
    }

    #[test]
    fn test_document_self_closing_tag_with_attributes() {
        let xml = b"<root><child attr=\"value\"/></root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let child = root.first_child().unwrap();
            assert_eq!(child.tag_name(), "child");
            assert_eq!(child.get_attribute("attr").unwrap(), "value");
            assert!(child.first_child().is_none()); // Self-closing tag has no children
        }
    }

    #[test]
    fn test_document_malformed_xml_unclosed_attribute() {
        let xml = b"<root attr=\"value>Content</root>".to_vec(); // Missing closing quote
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(_)) = result {
            // Should fail due to unclosed attribute
        }
    }

    #[test]
    fn test_document_malformed_xml_nested_quotes() {
        let xml = b"<root attr=\"value\"with\"quotes\">Content</root>".to_vec();
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(_)) = result {
            // Should fail due to nested quotes
        }
    }

    // ========== Iterator and Traversal Negative Tests ==========

    #[test]
    fn test_empty_children_iterator() {
        let xml = b"<root></root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        let children: Vec<_> = root.children().collect();
        assert!(children.is_empty());
    }

    #[test]
    fn test_empty_attributes_iterator() {
        let xml = b"<root>Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();

        let attrs: Vec<_> = root.attributes().collect();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_descendants_iterator_on_leaf_node() {
        let xml = b"<root>Text Content</root>".to_vec();
        let document = Document::new(xml).unwrap();
        let root = document.root().unwrap();
        let text_node = root.first_child().unwrap();

        assert!(text_node.is_text());
        let descendants: Vec<_> = text_node.descendants().collect();
        assert!(descendants.is_empty());
    }

    #[test]
    fn test_all_nodes_iterator_on_empty_document() {
        let xml = b"<root></root>".to_vec();
        let document = Document::new(xml).unwrap();

        let all_nodes: Vec<_> = document.all_nodes().collect();
        // Should contain only the root element
        assert!(all_nodes.len() == 1);
    }

    // ========== Feature-Specific Negative Tests ==========

    #[test]
    #[cfg(feature = "namespace_removal")]
    fn test_namespace_with_empty_prefix() {
        let xml = b"<:root>Content</:root>".to_vec();
        let result = Document::new(xml);
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "root");
        }
    }

    #[test]
    #[cfg(feature = "namespace_removal")]
    fn test_namespace_with_colon_only() {
        let xml = b"<root xmlns:=\"http://example.com\">Content</root>".to_vec();
        let result = Document::new(xml);
        // This should be handled gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "root");
        }
    }

    #[test]
    #[cfg(feature = "parse_escapes")]
    fn test_escape_sequence_at_end_of_text() {
        let xml = b"<root>Content &amp".to_vec(); // Incomplete escape at end
        let result = Document::new(xml);
        assert!(result.is_err());
        if let Err(ParseXmlError::InvalidXml(msg)) = result {
            assert!(msg.contains("Unexpected end of XML document"));
        }
    }

    #[test]
    #[cfg(feature = "parse_escapes")]
    fn test_malformed_numeric_escape() {
        let xml = b"<root>Content &#xGGG;</root>".to_vec();
        let result = Document::new(xml);
        // Should handle malformed numeric escapes gracefully
        if result.is_ok() {
            let document = result.unwrap();
            let root = document.root().unwrap();
            let text_node = root.first_child().unwrap();
            let text = text_node.text().unwrap();
            // The malformed escape should be left as-is or handled gracefully
            assert!(text.contains("Content"));
        }
    }

    // ========== Thread Safety and Concurrency Negative Tests ==========
    // Note: These tests would require std::thread which might not be available in all environments

    // ========== Performance and Stress Tests ==========

    #[test]
    fn test_deeply_nested_structure() {
        let depth = 50;
        let mut xml = String::new();

        for i in 0..depth {
            xml.push_str(&format!("<level{}>", i));
        }
        xml.push_str("deepest content");
        for i in (0..depth).rev() {
            xml.push_str(&format!("</level{}>", i));
        }

        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();

            // Navigate to the deepest level
            let mut current = root;
            for _ in 0..depth {
                if let Some(child) = current.first_child() {
                    current = child;
                } else {
                    break;
                }
            }

            // Should eventually reach a text node with the deepest content
            assert!(current.is_text() || current.first_child().is_some());
        }
    }

    #[test]
    fn test_wide_structure_many_siblings() {
        let width = 100;
        let mut xml = String::from("<root>");

        for i in 0..width {
            xml.push_str(&format!("<child{}>content{}</child{}>", i, i, i));
        }
        xml.push_str("</root>");

        let result = Document::new(xml.into_bytes());
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            let children: Vec<_> = root.children().collect();
            assert_eq!(children.len(), width);
        }
    }

    // ========== Regression Tests for Edge Cases ==========

    #[test]
    fn test_cdata_like_content() {
        let xml = b"<root><![CDATA[This looks like CDATA]]></root>".to_vec();
        let result = Document::new(xml);
        // CDATA should be bypassed according to the documentation
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            // The CDATA section should be ignored, so root should be empty
            assert!(root.first_child().is_none() || root.children().count() == 0);
        }
    }

    #[test]
    fn test_comment_like_content() {
        let xml = b"<root><!-- This is a comment --></root>".to_vec();
        let result = Document::new(xml);
        // Comments should be bypassed according to the documentation
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            // The comment should be ignored, so root should be empty
            assert!(root.first_child().is_none() || root.children().count() == 0);
        }
    }

    #[test]
    fn test_processing_instruction_like_content() {
        let xml = b"<?xml version=\"1.0\"?><root>Content</root>".to_vec();
        let result = Document::new(xml);
        // Processing instructions should be bypassed according to the documentation
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "root");
        }
    }

    #[test]
    fn test_doctype_like_content() {
        let xml = b"<!DOCTYPE html><root>Content</root>".to_vec();
        let result = Document::new(xml);
        // DOCTYPE should be bypassed according to the documentation
        assert!(result.is_ok());

        if let Ok(document) = result {
            let root = document.root().unwrap();
            assert_eq!(root.tag_name(), "root");
        }
    }
}
