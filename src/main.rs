use std::path::Path;

use clap::{Parser, command};
use image::GenericImageView;
use lib::{
    image_utils::save_images,
    pdf::{PdfError, PdfPageText, create_pdfium},
};

#[derive(Parser)]
#[command(name = "pdf_diff")]
#[command(about = "A tool for comparing PDF documents by generating visual diffs")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to the PDF file
    #[arg(short = 'p', long = "pdf", help = "Path to the PDF file")]
    pdf: String,

    /// Output directory for diff images
    #[arg(
        short = 'd',
        long = "debug",
        default_value = "false",
        help = "create debug output with bounding boxes"
    )]
    debug: bool,

    /// DPI for rendering (higher = better quality, slower processing)
    #[arg(long = "dpi", default_value = "300", help = "DPI for PDF rendering")]
    dpi: f32,

    /// Verbose output
    #[arg(short = 'v', long = "verbose", help = "Enable verbose output")]
    verbose: bool,
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

    if args.verbose {
        println!("Creating PDFium instance...");
    }
    if args.debug {
        match create_debug_output(
            &pdfium,
            path_pdf,
            Path::new("output").join("bbox_output.pdf").as_path(),
            args.dpi,
        ) {
            Ok(()) => println!("Successfully created bounding box images"),
            Err(e) => {
                eprintln!("Error creating bounding box images: {}", e);
                std::process::exit(1);
            }
        }
    }

    print!("Bench mark");
    let mut total_pages = 0;
    let start = std::time::Instant::now();
    let folder_path = Path::new("/Users/chris/code/pdf_text_extract/samples");
    let pdf_diles = std::fs::read_dir(folder_path).unwrap();
    for pdf_file in pdf_diles {
        let pdf_path = pdf_file.unwrap().path();
        if pdf_path.extension().unwrap_or_default() == "pdf" {
            let start = std::time::Instant::now();
            let pdf = match lib::pdf::load_pdf_document(&pdfium, &pdf_path) {
                Ok(doc) => doc,
                Err(e) => {
                    eprintln!("Error loading PDF document {:?}: {}", pdf_path, e);
                    continue;
                }
            };
            for (index, page) in pdf.pages().iter().enumerate() {
                let bbox = lib::pdf::get_text_bounding_box_from_page(&page, index);
                let paragrpahs = match bbox {
                    Ok(bbox) => lib::chunker::reduce_bbox_to_paragraphs(&bbox),
                    Err(e) => {
                        eprintln!(
                            "Error extracting bounding boxes from page {}: {}",
                            index + 1,
                            e
                        );
                        continue;
                    }
                };
                total_pages += 1;
            }

            // let _ = create_debug_output(&pdfium, &pdf_path, Path::new("output"), args.dpi);
            let duration = start.elapsed();
            println!(
                "Processed {:?} in: {:?}",
                pdf_path.file_name().unwrap(),
                duration
            );
        }
    }
    let duration = start.elapsed();
    println!("Total time elapsed: {:?}", duration);
    println!("Total pages processed: {}", total_pages);
    print!(
        "Pages per second: {} ms",
        total_pages as f64 / duration.as_secs_f64()
    );
}

fn create_debug_output(
    pdfium_context: &pdfium_render::prelude::Pdfium,
    pdf_path: &Path,
    output_path: &Path,
    dpi: f32,
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

    let text_with_bboxes: Vec<Vec<PdfPageText>> = document
        .pages()
        .iter()
        .enumerate()
        .map(|(i, page)| {
            lib::pdf::get_text_bounding_box_from_page(&page, i.into())
                .unwrap_or_else(|_| Vec::new())
        })
        .collect();
    let mut final_images = Vec::new();

    for i in 0..page_images.len() {
        println!("Processing page {} of {}", i + 1, page_images.len());

        let mut image = page_images[i].clone();
        let page_infos = &text_with_bboxes[i];
        let paragraphs = lib::chunker::reduce_bbox_to_paragraphs(page_infos);

        let (img_width, img_height) = image.dimensions();

        println!("  Image dimensions: {}x{}", img_width, img_height);
        println!(
            "  Found {} text segments on page {}",
            page_infos.len(),
            i + 1
        );

        for (j, page_info) in paragraphs.iter().enumerate() {
            let bbox = &page_info.bounding_box;

            // Debug: show original PDF coordinates
            println!(
                "    Text '{}': PDF coords ({:.1}, {:.1}, {:.1}, {:.1})",
                page_info.text.chars().take(10).collect::<String>(),
                bbox.x,
                bbox.y,
                bbox.width,
                bbox.height
            );

            // Scale to image coordinates
            let x = (bbox.x * scale) as u32;
            let y = (bbox.y * scale) as u32;
            let w = (bbox.width * scale) as u32;
            let h = (bbox.height * scale) as u32;

            println!("    Drawing rect {}: ({}, {}, {}, {})", j + 1, x, y, w, h);

            // Bounds check before drawing
            let (img_width, img_height) = image.dimensions();
            if x < img_width && y < img_height && w > 0 && h > 0 {
                let rect = lib::image_utils::draw_rect(&image, (x, y, w, h), false);
                image = rect;
            } else {
                println!("    Skipping invalid rectangle bounds");
            }
        }

        final_images.push(image);
        println!("  Completed page {}", i + 1);
    }

    save_images(final_images, "bbox_output", output_path.to_str().unwrap()).map_err(|e| {
        PdfError {
            message: format!("Failed to save images: {}", e),
        }
    })?;

    Ok(())
}
