use markup5ever_rcdom::RcDom;
use xml5ever::{
    driver::parse_document,
    tendril::{SliceExt, TendrilSink},
};

fn main() {
    let contents = std::fs::read("large.xhtml");

    assert!(contents.is_ok(), "Failed to read file: {:?}", "large.xhtml");

    let data = contents.unwrap().to_tendril();

    let start_time = std::time::Instant::now();

    let document: RcDom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .from_iter(std::iter::once(data));

    let duration = start_time.elapsed();
    println!("{}", duration.as_nanos());

}
