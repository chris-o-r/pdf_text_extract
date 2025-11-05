use std::path::Path;

use clap::{Parser, command};
use lib::{
    image_utils::save_images,
    models::{ChunkingStrategy, PdfPageText},
    pdf::{PdfError, create_pdfium},
};

#[derive(Parser)]
#[command(name = "pdf_diff")]
#[command(about = "A tool for comparing PDF documents by generating visual diffs")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to the PDF file
    #[arg(short = 'p', long = "pdf", help = "Path to the PDF file")]
    pdf: String,

    #[arg(
        short = 'c',
        default_value = "paragraph",
        long = "chunking",
        help = "Chunking strategy for text extraction"
    )]
    chunking_strategy: ChunkingStrategy,

    #[arg(
        long = "percentile",
        default_value = "85.0",
        help = "The percentile for paragraph threshold calculation, used when chunking strategy is paragraph",
        required_if_eq("chunking_strategy", "Paragraph")
    )]
    percentile: f32,

    /// Output directory for diff images
    #[arg(
        short = 'd',
        long = "debug",
        default_value = "false",
        help = "create debug output with bounding boxes"
    )]
    debug: bool,

    #[arg(
        short = 'b',
        long = "benchmark",
        default_value = "false",
        help = "create debug output with bounding boxes"
    )]
    benchmark: bool,

    #[arg(
        short = 'o',
        long = "output",
        default_value = "false",
        help = "save output as json"
    )]
    output: bool,
}

fn main() {
    let pdfium = match create_pdfium() {
        Ok(pdfium) => pdfium,
        Err(e) => {
            eprintln!("Error creating PDFium instance: {}", e);
            std::process::exit(1);
        }
    };

    let args = Args::parse();

    let path_pdf = Path::new(&args.pdf);

    // Validate input files exist
    if !path_pdf.exists() {
        eprintln!("Error: PDF file does not exist: {}", args.pdf);
        std::process::exit(1);
    }

    if args.debug {
        match create_debug_output(
            &pdfium,
            path_pdf,
            Path::new("output"),
            300.0,
            args.percentile,
        ) {
            Ok(()) => println!("Successfully created bounding box images"),
            Err(e) => {
                eprintln!("Error creating bounding box images: {}", e);
                std::process::exit(1);
            }
        }
    }

    let start_time = std::time::Instant::now();

    let pdf = match lib::pdf::load_pdf_document(&pdfium, path_pdf) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("Error loading PDF document: {}", e);
            std::process::exit(1);
        }
    };
    let mut bboxes = match lib::pdf::get_bounding_box_for_pdf(&pdf) {
        Ok(bbox) => bbox,
        Err(e) => {
            eprintln!("Error extracting bounding boxes from PDF: {}", e);
            std::process::exit(1);
        }
    };

    if args.chunking_strategy == ChunkingStrategy::Paragraph {
        bboxes = lib::chunker::reduce_bbox_to_paragraphs(&bboxes, args.percentile);
    }

    if args.output {
        let json_output = serde_json::to_string_pretty(&bboxes).unwrap();
        let output_path = Path::new("output").join("output.json");
        std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
        std::fs::write(&output_path, json_output).unwrap();
        println!("Saved output to {:?}", output_path);
    } else {
        let json_output = serde_json::to_string_pretty(&bboxes).unwrap();
        println!("{}", json_output);
    }

    if args.benchmark {
        let total_duration = start_time.elapsed();
        println!("Total processing time: {:?}", total_duration);

        let total_pages = pdf.pages().len();
        println!("Total pages processed: {}", total_pages);
        println!(
            "Times to process per page: {:.2} ms",
            total_duration.as_millis() as f64 / total_pages as f64
        );
        println!("Paragraphs created {}", bboxes.len());
    }
}

fn create_debug_output(
    pdfium_context: &pdfium_render::prelude::Pdfium,
    pdf_path: &Path,
    output_path: &Path,
    dpi: f32,
    percentile: f32,
) -> Result<(), PdfError> {
    // Scale from PDF points (72 DPI) to target DPI
    let scale = dpi / 72.0;
    println!("Using scale factor: {} (DPI: {})", scale, dpi);

    let document = pdfium_context
        .load_pdf_from_file(pdf_path, None)
        .map_err(|e| PdfError {
            message: format!("Failed to load old PDF file: {:?}", e),
        })?;

    let page_images = lib::pdf::create_images_from_pdf(&document, dpi)?;

    let text_with_bboxes = lib::pdf::get_bounding_box_for_pdf(&document)?;
    let mut final_images = Vec::new();

    let name = pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    for (i, image) in page_images.iter().enumerate() {
        let mut image = image.clone();
        let page_infos = text_with_bboxes
            .iter()
            .filter(|bbox| bbox.page_index == i)
            .cloned()
            .collect::<Vec<PdfPageText>>();
        let paragraphs = lib::chunker::reduce_bbox_to_paragraphs(&page_infos, percentile);

        for page_info in paragraphs.iter() {
            let bbox = &page_info.bounding_box;

            // Scale to image coordinates
            let x = (bbox.x * scale) as u32;
            let y = (bbox.y * scale) as u32;
            let w = (bbox.width * scale) as u32;
            let h = (bbox.height * scale) as u32;

            // Bounds check before drawing
            image = lib::image_utils::draw_rects(&image, &[(x, y, w, h)], false).unwrap();
        }

        final_images.push(image);
        println!("  Completed page {}", i + 1);
    }

    save_images(final_images, name, output_path.to_str().unwrap()).map_err(|e| PdfError {
        message: format!("Failed to save images: {}", e),
    })?;

    Ok(())
}
