use image::ImageFormat;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::DynamicImage;

pub fn save_images(
    images: Vec<DynamicImage>,
    pdf_title: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(output_dir).join(pdf_title);
    std::fs::create_dir_all(&path)?;

    for (i, img) in images.iter().enumerate() {
        let output_path = path.join(format!("{}_{}.png", pdf_title, i + 1));
        let file = File::create(&output_path)?;
        let w = BufWriter::new(file);
        img.write_to(&mut BufWriter::new(w), ImageFormat::Png)?;
        println!("Saved diff image to {}", output_path.display());
    }

    Ok(())
}

pub fn draw_rect(
    images: &DynamicImage,
    (x, y, w, h): (u32, u32, u32, u32),
    fill: bool,
) -> DynamicImage {
    let mut img = images.to_rgba8();
    for i in x..(x + w) {
        for j in y..(y + h) {
            if i >= img.width() || j >= img.height() {
                continue;
            }
            if fill || i == x || i == x + w - 1 || j == y || j == y + h - 1 {
                img.put_pixel(i, j, image::Rgba([255, 0, 0, 255]));
            }
        }
    }

    DynamicImage::ImageRgba8(img)
}

pub fn draw_rects(
    image: &DynamicImage,
    rects: &[(u32, u32, u32, u32)],
    fill: bool,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let mut img = image.to_rgba8();
    for (x, y, w, h) in rects {
        for i in *x..(*x + *w) {
            for j in *y..(*y + *h) {
                if i >= img.width() || j >= img.height() {
                    return Err(format!(
                        "Rectangle ({}, {}, {}, {}) exceeds image bounds ({}, {})",
                        x,
                        y,
                        w,
                        h,
                        img.width(),
                        img.height()
                    )
                    .into());
                }
                if fill || i == *x || i == *x + *w - 1 || j == *y || j == *y + *h - 1 {
                    img.put_pixel(i, j, image::Rgba([255, 0, 0, 255]));
                }
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(img))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba, RgbaImage};
    use std::fs;
    use std::path::Path;

    // Helper function to create a solid color image
    fn create_solid_color_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        let img: RgbaImage = ImageBuffer::from_pixel(width, height, color);
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_save_images() {
        let test_dir = "test_output";
        let pdf_title = "test_pdf";
        let img1 = create_solid_color_image(10, 10, Rgba([255, 0, 0, 255]));
        let img2 = create_solid_color_image(10, 10, Rgba([0, 255, 0, 255]));

        let images = vec![img1, img2];

        // Clean up any existing test directory
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).ok();
        }

        let result = save_images(images, pdf_title, test_dir);
        assert!(result.is_ok(), "save_images should succeed");

        // Check that files were created
        assert!(Path::new(&format!("{}/{}_1.png", test_dir, pdf_title)).exists());
        assert!(Path::new(&format!("{}/{}_2.png", test_dir, pdf_title)).exists());

        // Clean up
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_save_images_empty_vec() {
        let test_dir = "test_output_empty";
        let pdf_title = "test_pdf";
        let images = vec![];

        // Clean up any existing test directory
        if Path::new(test_dir).exists() {
            fs::remove_dir_all(test_dir).ok();
        }

        let result = save_images(images, pdf_title, test_dir);
        assert!(
            result.is_ok(),
            "save_images should succeed with empty vector"
        );

        // Directory should be created even with no images
        assert!(Path::new(test_dir).exists());

        // Clean up
        fs::remove_dir_all(test_dir).ok();
    }

    #[test]
    fn test_save_images_creates_directory() {
        let test_dir = "test_output_new_dir/subdir";
        let img = create_solid_color_image(5, 5, Rgba([100, 100, 100, 255]));
        let images = vec![img];

        let pdf_title = "test_pdf";
        // Ensure directory doesn't exist
        if Path::new("test_output_new_dir").exists() {
            fs::remove_dir_all("test_output_new_dir").ok();
        }

        let result = save_images(images, pdf_title, test_dir);
        assert!(
            result.is_ok(),
            "save_images should create nested directories"
        );

        assert!(Path::new(&format!("{}/{}_1.png", test_dir, pdf_title)).exists());

        // Clean up
        fs::remove_dir_all("test_output_new_dir").ok();
    }
    #[test]
    fn test_draw_rect_filled() {
        let img = create_solid_color_image(50, 50, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (10, 10, 20, 15), true);

        let rgba = result.to_rgba8();

        // Check that all pixels in the rectangle are red
        for y in 10..25 {
            for x in 10..30 {
                let pixel = rgba.get_pixel(x, y);
                assert_eq!(
                    pixel,
                    &Rgba([255, 0, 0, 255]),
                    "Pixel at ({}, {}) should be red",
                    x,
                    y
                );
            }
        }

        // Check that pixels outside the rectangle are unchanged
        assert_eq!(rgba.get_pixel(9, 10), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(30, 10), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(10, 9), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(10, 25), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_draw_rect_outline_only() {
        let img = create_solid_color_image(50, 50, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (10, 10, 20, 15), false);

        let rgba = result.to_rgba8();

        // Check corners are red
        assert_eq!(rgba.get_pixel(10, 10), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(29, 10), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(10, 24), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(29, 24), &Rgba([255, 0, 0, 255]));

        // Check edges are red
        assert_eq!(rgba.get_pixel(15, 10), &Rgba([255, 0, 0, 255])); // top edge
        assert_eq!(rgba.get_pixel(15, 24), &Rgba([255, 0, 0, 255])); // bottom edge
        assert_eq!(rgba.get_pixel(10, 15), &Rgba([255, 0, 0, 255])); // left edge
        assert_eq!(rgba.get_pixel(29, 15), &Rgba([255, 0, 0, 255])); // right edge

        // Check interior is unchanged (white)
        assert_eq!(rgba.get_pixel(15, 15), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(20, 20), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_draw_rect_at_origin() {
        let img = create_solid_color_image(30, 30, Rgba([0, 255, 0, 255])); // Green background
        let result = draw_rect(&img, (0, 0, 10, 10), true);

        let rgba = result.to_rgba8();

        // Check top-left corner and some filled area
        assert_eq!(rgba.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(5, 5), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(9, 9), &Rgba([255, 0, 0, 255]));

        // Check area outside rectangle is unchanged
        assert_eq!(rgba.get_pixel(10, 10), &Rgba([0, 255, 0, 255]));
        assert_eq!(rgba.get_pixel(15, 15), &Rgba([0, 255, 0, 255]));
    }

    #[test]
    fn test_draw_rect_exceeds_bounds() {
        let img = create_solid_color_image(20, 20, Rgba([0, 0, 255, 255])); // Blue background
        let result = draw_rect(&img, (15, 15, 10, 10), true); // Rectangle extends beyond image

        let rgba = result.to_rgba8();

        // Check that pixels within bounds are drawn
        assert_eq!(rgba.get_pixel(15, 15), &Rgba([255, 0, 0, 255]));
        assert_eq!(rgba.get_pixel(19, 19), &Rgba([255, 0, 0, 255]));

        // Check that original pixels outside the attempted rectangle are unchanged
        assert_eq!(rgba.get_pixel(14, 14), &Rgba([0, 0, 255, 255]));
        assert_eq!(rgba.get_pixel(10, 10), &Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn test_draw_rect_completely_out_of_bounds() {
        let img = create_solid_color_image(10, 10, Rgba([128, 128, 128, 255])); // Gray background
        let result = draw_rect(&img, (20, 20, 5, 5), true); // Completely outside

        let rgba_original = img.to_rgba8();
        let rgba_result = result.to_rgba8();

        // Image should remain unchanged
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    rgba_result.get_pixel(x, y),
                    rgba_original.get_pixel(x, y),
                    "Pixel at ({}, {}) should be unchanged",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_draw_rect_single_pixel() {
        let img = create_solid_color_image(10, 10, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (5, 5, 1, 1), true);

        let rgba = result.to_rgba8();

        // Only one pixel should be red
        assert_eq!(rgba.get_pixel(5, 5), &Rgba([255, 0, 0, 255]));

        // All other pixels should remain white
        assert_eq!(rgba.get_pixel(4, 5), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(6, 5), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(5, 4), &Rgba([255, 255, 255, 255]));
        assert_eq!(rgba.get_pixel(5, 6), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn test_draw_rect_zero_dimensions() {
        let img = create_solid_color_image(10, 10, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (5, 5, 0, 0), true);

        let rgba_original = img.to_rgba8();
        let rgba_result = result.to_rgba8();

        // Image should remain unchanged when drawing zero-sized rectangle
        for y in 0..10 {
            for x in 0..10 {
                assert_eq!(
                    rgba_result.get_pixel(x, y),
                    rgba_original.get_pixel(x, y),
                    "Pixel at ({}, {}) should be unchanged",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_draw_rect_thin_line_horizontal() {
        let img = create_solid_color_image(20, 20, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (5, 10, 10, 1), true); // Thin horizontal line

        let rgba = result.to_rgba8();

        // Check horizontal line is drawn
        for x in 5..15 {
            assert_eq!(rgba.get_pixel(x, 10), &Rgba([255, 0, 0, 255]));
        }

        // Check pixels above and below are unchanged
        for x in 5..15 {
            assert_eq!(rgba.get_pixel(x, 9), &Rgba([255, 255, 255, 255]));
            assert_eq!(rgba.get_pixel(x, 11), &Rgba([255, 255, 255, 255]));
        }
    }

    #[test]
    fn test_draw_rect_thin_line_vertical() {
        let img = create_solid_color_image(20, 20, Rgba([255, 255, 255, 255])); // White background
        let result = draw_rect(&img, (10, 5, 1, 10), true); // Thin vertical line

        let rgba = result.to_rgba8();

        // Check vertical line is drawn
        for y in 5..15 {
            assert_eq!(rgba.get_pixel(10, y), &Rgba([255, 0, 0, 255]));
        }

        // Check pixels left and right are unchanged
        for y in 5..15 {
            assert_eq!(rgba.get_pixel(9, y), &Rgba([255, 255, 255, 255]));
            assert_eq!(rgba.get_pixel(11, y), &Rgba([255, 255, 255, 255]));
        }
    }

    #[test]
    fn test_draw_rect_outline_vs_filled() {
        let img1 = create_solid_color_image(20, 20, Rgba([255, 255, 255, 255])); // White background
        let img2 = img1.clone();

        let filled = draw_rect(&img1, (5, 5, 10, 8), true);
        let outline = draw_rect(&img2, (5, 5, 10, 8), false);

        let rgba_filled = filled.to_rgba8();
        let rgba_outline = outline.to_rgba8();

        // Interior should be different between filled and outline
        assert_eq!(rgba_filled.get_pixel(7, 7), &Rgba([255, 0, 0, 255])); // Red for filled
        assert_eq!(rgba_outline.get_pixel(7, 7), &Rgba([255, 255, 255, 255])); // White for outline

        // Edges should be the same (red) for both
        assert_eq!(rgba_filled.get_pixel(5, 5), rgba_outline.get_pixel(5, 5)); // Top-left corner
        assert_eq!(
            rgba_filled.get_pixel(14, 12),
            rgba_outline.get_pixel(14, 12)
        ); // Bottom-right corner
    }
}
