#[cfg(test)]
mod xhtml_parser_tests {
    use xhtml_parser::document::Document;
    use xhtml_parser::node::Node;

    use test_support::unit_test::UnitTest;
    use timelapse::{profile_end_print, profile_start, TimeLapse};

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

    #[test]
    fn test_speed_test() {
        let unit_test = UnitTest::new("speed_test");

        println!("Speed Test Case Folder: {:?}", unit_test.test_case_folder());

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("Speed Testing File: {:?}", file_name);

                let contents = std::fs::read(&file);
                assert!(contents.is_ok(), "Failed to read file: {:?}", file_name);

                profile_start!(speed_testing);
                let document = Document::new(contents.unwrap());
                profile_end_print!(speed_testing);

                assert!(
                    document.is_ok(),
                    "Failed to process xml file: {:?} : {:?}",
                    file_name,
                    document.err().unwrap()
                );

                let doc = document.unwrap();

                println!("Node count: {}", doc.last_node_idx());

                profile_start!(formatting_data);
                let data = format!("{:#?}", doc);
                profile_end_print!(formatting_data);

                assert!(unit_test.check_result_with_file(&data, &file_name));
            }
        }
    }

    #[test]
    #[cfg(feature = "trim_pcdata")]
    fn test_trim_pcdata() {
        let unit_test = UnitTest::new("trim_pcdata");

        println!(
            "trim_pcdata Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("trim_pcdata Testing File: {:?}", file_name);

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

    #[test]
    #[cfg(feature = "keep_ws_only_pcdata")]
    fn test_keep_ws_only_pcdata() {
        let unit_test = UnitTest::new("keep_ws_only_pcdata");

        println!(
            "keep_ws_only_pcdata Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("keep_ws_only_pcdata Testing File: {:?}", file_name);

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

    #[test]
    #[cfg(feature = "namespace_removal")]
    fn test_namespace_removal() {
        let unit_test = UnitTest::new("namespace_removal");

        println!(
            "namespace_removal Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("namespace_removal Testing File: {:?}", file_name);

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

    #[test]
    #[cfg(feature = "parse_escapes")]
    fn test_parse_escapes() {
        let unit_test = UnitTest::new("parse_escapes");

        println!(
            "parse_escapes Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("parse_escapes Testing File: {:?}", file_name);

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

    #[test]
    #[cfg(not(any(
        feature = "namespace_removal",
        feature = "parse_escapes",
        feature = "trim_pcdata",
        feature = "keep_ws_only_pcdata",
        feature = "use_cstr"
    )))]
    fn test_parse_no_feature() {
        let unit_test = UnitTest::new("no_feature");

        println!(
            "no_feature Test Case Folder: {:?}",
            unit_test.test_case_folder()
        );

        let files = unit_test.get_test_case_file_paths().unwrap();

        for file in files {
            let file_name = file.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".xhtml") {
                println!("no_feature Testing File: {:?}", file_name);

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

    #[test]
    fn test_descendant_iterator() {
        let xml_data = b"<root><child>Text</child><totototo/></root>".to_vec();
        let document = Document::new(xml_data).unwrap();
        let root_node = document.root().unwrap();
        let descendants: Vec<Node> = document.descendants(root_node.idx()).collect();

        assert_eq!(descendants.len(), 3); // child, Text, and totototo
        assert!(descendants[0].is("child"));
        assert_eq!(descendants[1].text().unwrap(), "Text");
        assert!(descendants[2].is("totototo"));
    }
}
