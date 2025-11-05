use std::path::Path;

#[allow(unused_imports)]
use image::{DynamicImage, GenericImageView};
use pdfium_render::prelude::{PdfDocument, PdfRenderConfig, Pdfium};
use std::error::Error;
use std::fmt;

use crate::models::{BoundingBox, PdfPageText};

#[derive(Debug)]
pub struct PdfError {
    pub message: String,
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for PdfError {}

pub fn create_pdfium() -> Result<Pdfium, PdfError> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            "./pdfium-mac-arm64/lib/",
        ))
        .map_err(|e| PdfError {
            message: format!("Failed to bind to PDFium library: {:?}", e),
        })?,
    );
    Ok(pdfium)
}

pub fn load_pdf_document<'a>(
    pdfium: &'a Pdfium,
    pdf_path: &Path,
) -> Result<PdfDocument<'a>, PdfError> {
    if !pdf_path.exists() {
        return Err(PdfError {
            message: format!("PDF file does not exist: {:?}", pdf_path),
        });
    }

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(|e| PdfError {
            message: format!("Failed to load old PDF file: {:?}", e),
        })?;

    Ok(document)
}

pub fn create_images_from_pdf(
    document: &PdfDocument,
    dpi: f32,
) -> Result<Vec<DynamicImage>, PdfError> {
    document
        .pages()
        .iter()
        .map(|page| get_image_from_page(&page, dpi))
        .collect()
}

fn get_image_from_page(
    page: &pdfium_render::prelude::PdfPage,
    dpi: f32,
) -> Result<DynamicImage, PdfError> {
    let render_config = PdfRenderConfig::new()
        .set_target_width((page.width().value * dpi / 72.0).round() as i32)
        .set_maximum_height((page.height().value * dpi / 72.0).round() as i32);

    Ok(page
        .render_with_config(&render_config)
        .map_err(|e| PdfError {
            message: format!("Failed to render page to image: {:?}", e),
        })?
        .as_image())
}

pub fn get_bounding_box_for_pdf(document: &PdfDocument) -> Result<Vec<PdfPageText>, PdfError> {
    document
        .pages()
        .iter()
        .enumerate()
        .map(|(index, page)| get_text_bounding_box_from_page(&page, index))
        .collect::<Result<Vec<Vec<PdfPageText>>, PdfError>>()
        .map(|vec_of_vecs| vec_of_vecs.into_iter().flatten().collect())
}

pub fn get_text_bounding_box_from_page(
    page: &pdfium_render::prelude::PdfPage,
    index: usize,
) -> Result<Vec<PdfPageText>, PdfError> {
    let text = page.text().map_err(|e| PdfError {
        message: format!("Failed to extract text from page: {:?}", e),
    })?;

    Ok(text
        .segments()
        .iter()
        .map(|segment| {
            let quads = segment.bounds();
            let page_height = page.height().value;

            let bounding_box = BoundingBox::new(
                quads.left().value,
                page_height - quads.top().value, // Convert from PDF coords to image coords
                quads.width().value,
                quads.height().value, // Actual height, not coordinate
            );
            PdfPageText {
                page_index: index,
                text: segment.text().to_string(),
                bounding_box,
            }
        })
        .collect::<Vec<PdfPageText>>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_create_pdfium() {
        let result = create_pdfium();
        assert!(
            result.is_ok(),
            "Failed to create Pdfium instance: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_load_pdf_documents() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let path = Path::new("./samples/old.pdf");

        let result = load_pdf_document(&pdfium, path);
        assert!(
            result.is_ok(),
            "Failed to load PDF documents: {:?}",
            result.err()
        );

        let doc = result.unwrap();
        assert!(doc.pages().len() > 0, "document should have pages");
    }

    #[test]
    fn test_load_pdf_documents_nonexistent_file() {
        let pdfium = create_pdfium().unwrap();
        let new_path = Path::new("./samples/does_not_exist.pdf");

        let result = load_pdf_document(&pdfium, new_path);
        assert!(result.is_err(), "Should fail for nonexistent file");

        let error = result.err().unwrap();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn test_create_images_from_pdf() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let path = Path::new("./samples/old.pdf");

        let doc = load_pdf_document(&pdfium, path).unwrap();

        let result = create_images_from_pdf(&doc, 300.0);
        assert!(
            result.is_ok(),
            "Failed to create images from PDF: {:?}",
            result.err()
        );

        let images = result.unwrap();
        assert!(images.len() > 0, "Should generate at least one image pair");

        // Check that we have valid image data
        for (i, img) in images.iter().enumerate() {
            let (width, height) = img.dimensions();
            assert!(
                width > 0 && height > 0,
                "Image {} should have positive dimensions",
                i
            );
        }
    }

    #[test]
    fn test_get_image_from_page() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let pdf_path = Path::new("./samples/old.pdf");

        let doc = load_pdf_document(&pdfium, pdf_path).expect("Failed to load PDF document");

        let page = doc.pages().get(0).expect("Failed to get first page");
        let result = get_image_from_page(&page, 300.0);

        assert!(
            result.is_ok(),
            "Failed to render page to image: {:?}",
            result.err()
        );

        let image = result.unwrap();
        let (width, height) = image.dimensions();
        assert!(
            width > 0 && height > 0,
            "Image should have positive dimensions"
        );
        // Remove the width/height constraints since they depend on DPI and page size
    }

    #[test]
    fn test_get_text_from_page() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let pdf_path = Path::new("./samples/old.pdf");

        let doc = load_pdf_document(&pdfium, pdf_path).expect("Failed to load PDF document");

        let page = doc.pages().get(0).expect("Failed to get first page");
        let result = get_text_bounding_box_from_page(&page, 0);

        assert!(
            result.is_ok(),
            "Failed to extract text from page: {:?}",
            result.err()
        );

        let text = result.unwrap();
        print!("Extracted text: {:?}", text);
        assert!(
            !text.is_empty(),
            "Extracted text should not be empty for the first page"
        );
    }

    #[test]
    fn test_get_text_from_entire_document() {
        let pdfium = create_pdfium().expect("Failed to create Pdfium instance");
        let pdf_path = Path::new("./samples/1281082.pdf");

        let doc = load_pdf_document(&pdfium, pdf_path).expect("Failed to load PDF document");

        let mut all_text = Vec::<PdfPageText>::new();
        for index in 0..doc.pages().len() {
            let page = doc.pages().get(index).expect("Failed to get page");
            let page_text = get_text_bounding_box_from_page(&page, index.into())
                .expect("Failed to extract text from page");
            all_text.extend(page_text);
        }

        print!("Extracted text from entire document: {:?}", all_text);
        assert!(
            !all_text.is_empty(),
            "Extracted text should not be empty for the entire document"
        );
    }
}
