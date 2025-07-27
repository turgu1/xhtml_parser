use roxmltree::Document;

fn main() {
    let contents = std::fs::read_to_string("large.xhtml");
    // let file_name = "large.xhtml";
    // // Ensure the file exists and can be read
    // assert!(
    //     std::path::Path::new(file_name).exists(),
    //     "File does not exist: {}",
    //     file_name
    // );
    assert!(contents.is_ok(), "Failed to read file: {:?}", "large.xhtml");

    let data = contents.unwrap();
    let start_time = std::time::Instant::now();

    let document = Document::parse(data.as_str());

    let duration = start_time.elapsed();
    println!("{}", duration.as_nanos());

    assert!(
        document.is_ok(),
        "Failed to parse document: {:?}",
        document.err()
    );
}
