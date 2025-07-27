use xhtml_parser::document::Document;

fn main() {
    let contents = std::fs::read("large.xhtml");
    let file_name = "large.xhtml";
    // Ensure the file exists and can be read
    assert!(
        std::path::Path::new(file_name).exists(),
        "File does not exist: {}",
        file_name
    );
    assert!(contents.is_ok(), "Failed to read file: {:?}", file_name);

    let start_time = std::time::Instant::now();

    let document = Document::new(contents.unwrap());

    let duration = start_time.elapsed();
    println!("{}", duration.as_nanos());

    assert!(
        document.is_ok(),
        "Failed to parse document: {:?}",
        document.err()
    );
}
