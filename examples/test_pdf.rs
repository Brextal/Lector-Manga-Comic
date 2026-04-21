use lector_pdf::pdf_viewer::PdfViewer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("Usage: test_pdf <path>");

    println!("Testing PDF: {}", path);

    match PdfViewer::new(path) {
        Some(viewer) => println!("SUCCESS! Pages: {}", viewer.total_pages()),
        None => println!("FAILED to create viewer"),
    }
}
