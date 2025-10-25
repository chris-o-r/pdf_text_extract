use diff_img::lcs_diff;
use image::{DynamicImage, GenericImageView};
// Crop a DynamicImage to its non-white content (tolerant to near-white)
pub fn crop_to_content(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    // Tolerance for "white" (255,255,255,255). Allow a little off-white.
    let tolerance = 10u8;

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            // If not close to white, mark as content
            if !(pixel[0] >= 255 - tolerance
                && pixel[1] >= 255 - tolerance
                && pixel[2] >= 255 - tolerance
                && pixel[3] >= 255 - tolerance)
            {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    if found {
        img.crop_imm(min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
    } else {
        img.clone()
    }
}

pub fn save_images(
    images: Vec<DynamicImage>,
    pdf_title: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::ImageFormat;
    use std::fs::File;
    use std::io::BufWriter;

    std::fs::create_dir_all(output_dir)?;

    for (i, img) in images.iter().enumerate() {
        let output_path = format!("{}/{}_{}.png", output_dir, pdf_title, i + 1);
        let file = File::create(&output_path)?;
        let w = BufWriter::new(file);
        img.write_to(&mut BufWriter::new(w), ImageFormat::Png)?;
        println!("Saved diff image to {}", output_path);
    }

    Ok(())
}

pub fn diff_images(
    images: Vec<(Option<DynamicImage>, Option<DynamicImage>)>,
    sensitivity: f32,
) -> Result<Vec<DynamicImage>, Box<dyn std::error::Error>> {
    let mut diff = vec![];

    for (old_image, new_image) in &images {
        match (old_image, new_image) {
            (Some(old), Some(new)) => {
                let mut old = old.clone();
                let mut new = new.clone();

                let diff_ratio = diff_img::numerical::calculate_diff_ratio(&old, &new).unwrap();
                if diff_ratio > 0.0 {
                    let diff_image = lcs_diff(&mut old, &mut new, sensitivity)?;
                    diff.push(diff_image);
                }

                diff.push(new);
            }
            (None, Some(new)) => {
                diff.push(new.clone());
            }
            (Some(old), None) => {
                diff.push(old.clone());
            }
            (None, None) => {}
        }
    }

    Ok(diff)
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba, RgbaImage};
    use std::fs;
    use std::path::Path;

    // Helper function to create a test image with white background and colored content
    fn create_test_image_with_content(
        width: u32,
        height: u32,
        content_x: u32,
        content_y: u32,
        content_width: u32,
        content_height: u32,
    ) -> DynamicImage {
        let mut img: RgbaImage = ImageBuffer::new(width, height);

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }

        // Add colored content in specified area
        for y in content_y..(content_y + content_height).min(height) {
            for x in content_x..(content_x + content_width).min(width) {
                img.put_pixel(x, y, Rgba([100, 100, 100, 255])); // Gray content
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    // Helper function to create a solid color image
    fn create_solid_color_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        let img: RgbaImage = ImageBuffer::from_pixel(width, height, color);
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_crop_to_content_with_content() {
        let img = create_test_image_with_content(100, 100, 20, 30, 40, 25);
        let cropped = crop_to_content(&img);

        let (cropped_width, cropped_height) = cropped.dimensions();
        assert_eq!(
            cropped_width, 40,
            "Cropped width should match content width"
        );
        assert_eq!(
            cropped_height, 25,
            "Cropped height should match content height"
        );
    }

    #[test]
    fn test_crop_to_content_all_white() {
        let img = create_solid_color_image(100, 100, Rgba([255, 255, 255, 255]));
        let cropped = crop_to_content(&img);

        let (original_width, original_height) = img.dimensions();
        let (cropped_width, cropped_height) = cropped.dimensions();

        // Should return the original image since no content found
        assert_eq!(cropped_width, original_width);
        assert_eq!(cropped_height, original_height);
    }

    #[test]
    fn test_crop_to_content_near_white() {
        // Test with slightly off-white pixels (within tolerance)
        let img = create_solid_color_image(50, 50, Rgba([250, 250, 250, 255]));
        let cropped = crop_to_content(&img);

        let (original_width, original_height) = img.dimensions();
        let (cropped_width, cropped_height) = cropped.dimensions();

        // Should treat near-white as white and return original
        assert_eq!(cropped_width, original_width);
        assert_eq!(cropped_height, original_height);
    }

    #[test]
    fn test_crop_to_content_edge_content() {
        // Test content at the very edges
        let img = create_test_image_with_content(50, 50, 0, 0, 50, 50);
        let cropped = crop_to_content(&img);

        let (cropped_width, cropped_height) = cropped.dimensions();
        assert_eq!(cropped_width, 50);
        assert_eq!(cropped_height, 50);
    }

    #[test]
    fn test_diff_images_with_differences() {
        let img1 = create_solid_color_image(100, 100, Rgba([255, 0, 0, 255])); // Red
        let img2 = create_solid_color_image(100, 100, Rgba([0, 255, 0, 255])); // Green

        let images = vec![(Some(img1), Some(img2.clone()))];
        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        assert_eq!(result.len(), 2, "Should return diff image and new image");

        // Check that we got the new image as the last result
        let last_image = &result[1];
        assert_eq!(last_image.dimensions(), (100, 100));
    }

    #[test]
    fn test_diff_images_no_differences() {
        let img1 = create_solid_color_image(100, 100, Rgba([255, 0, 0, 255]));
        let img2 = img1.clone(); // Identical images

        let images = vec![(Some(img1), Some(img2.clone()))];
        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        // Should only return the new image (no diff because images are identical)
        assert_eq!(
            result.len(),
            1,
            "Should only return new image when no differences"
        );
    }

    #[test]
    fn test_diff_images_only_new() {
        let img = create_solid_color_image(100, 100, Rgba([255, 0, 0, 255]));

        let images = vec![(None, Some(img.clone()))];
        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        assert_eq!(result.len(), 1, "Should return only new image");
        assert_eq!(result[0].dimensions(), img.dimensions());
    }

    #[test]
    fn test_diff_images_only_old() {
        let img = create_solid_color_image(100, 100, Rgba([255, 0, 0, 255]));

        let images = vec![(Some(img.clone()), None)];
        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        assert_eq!(result.len(), 1, "Should return only old image");
        assert_eq!(result[0].dimensions(), img.dimensions());
    }

    #[test]
    fn test_diff_images_both_none() {
        let images = vec![(None, None)];
        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        assert_eq!(
            result.len(),
            0,
            "Should return empty vec when both images are None"
        );
    }

    #[test]
    fn test_diff_images_multiple_pairs() {
        let img1 = create_solid_color_image(50, 50, Rgba([255, 0, 0, 255]));
        let img2 = create_solid_color_image(50, 50, Rgba([0, 255, 0, 255]));
        let img3 = create_solid_color_image(50, 50, Rgba([0, 0, 255, 255]));

        let images = vec![(Some(img1), Some(img2.clone())), (None, Some(img3.clone()))];

        let result = diff_images(images, 0.12).expect("diff_images should succeed");

        // First pair: diff + new image = 2 images
        // Second pair: just new image = 1 image
        // Total = 3 images
        assert_eq!(result.len(), 3, "Should return correct number of images");
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
