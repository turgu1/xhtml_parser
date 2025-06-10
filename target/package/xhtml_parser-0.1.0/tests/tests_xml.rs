#[cfg(test)]
mod xhtml_parser_tests {
    use xhtml_parser::document::Document;

    use test_support::unit_test::UnitTest;

    #[test]
    fn test_accessing_tag_name() {
        let xml_data = b"<root><child>Text</child></root>".to_vec();
        let document = Document::new(xml_data).unwrap();
        let root_node = document.root().unwrap();
        let child_node = root_node.first_child().unwrap();
        assert_eq!(child_node.tag_name(), "child");
    }
    
    #[test]
    fn test_simple_xml_files() {
        let unit_test = UnitTest::new("simple_test");

        println!(
            "Simple Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("Simple Testing File: {:?}", file_name);

                let contents = std::fs::read(&file);
                assert!(contents.is_ok(), "Failed to read file: {:?}", file_name);
                let document = Document::new(contents.unwrap());

                assert!(
                    document.is_ok(),
                    "Failed to process xml file: {:?} : {:?}",
                    file_name,
                    document.err().unwrap()
                );

                let data = format!("{:#?}", document.unwrap());
                assert!(unit_test.check_result_with_file(&data, &file_name));
            }
        }
    }
}
