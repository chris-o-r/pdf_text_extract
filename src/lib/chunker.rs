use crate::models::PdfPageText;

pub fn reduce_bbox_to_paragraphs(bbox: &[PdfPageText], percentile: f32) -> Vec<PdfPageText> {
    let mut result: Vec<PdfPageText> = Vec::new();

    let mut current_paragraph: Option<PdfPageText> = None;

    let paragraph_threshold = calculate_paragraph_gap_threshold(bbox, percentile);

    for segment in bbox.iter().cloned() {
        if current_paragraph.is_none() {
            // Start the first paragraph with the first segment
            current_paragraph = Some(segment);
        } else {
            let current_paragraph_y = current_paragraph.as_ref().unwrap().bounding_box.y;

            // Calculate the vertical gap between the current paragraph and this segment
            let y_diff = (segment.bounding_box.y
                - (current_paragraph_y + current_paragraph.as_ref().unwrap().bounding_box.height))
                .abs();

            if y_diff <= paragraph_threshold {
                // Gap is small enough - combine this segment with the current paragraph
                current_paragraph = Some(current_paragraph.unwrap().combine(&segment));
            } else {
                // Gap is too large - finish current paragraph and start a new one
                result.push(current_paragraph.clone().unwrap());
                current_paragraph = Some(segment);
            }
        }
    }

    // Don't forget to add the final paragraph
    if let Some(paragraph) = current_paragraph {
        result.push(paragraph);
    }
    // Calculate the threshold distance for grouping segments into paragraphs

    reduce_short_paragraphs_to_parents(result, 25.0)
}

fn reduce_short_paragraphs_to_parents(
    paragraphs: Vec<PdfPageText>,
    length_percentile: f32,
) -> Vec<PdfPageText> {
    let lengths: Vec<usize> = paragraphs.iter().map(|p| p.text.len()).collect();
    let length_threshold = calculate_paragraph_length_threshold(&lengths, length_percentile);

    let mut result: Vec<PdfPageText> = Vec::new();

    for paragraph in paragraphs.into_iter() {
        let distance = paragraph.bounding_box.y - result.last().map_or(0.0, |p| p.bounding_box.y);
        if paragraph.text.len() < length_threshold as usize && distance < 50.0 {
            let last_paragraph = result.pop();

            if let Some(mut last) = last_paragraph {
                last = last.combine(&paragraph);
                result.push(last);
                continue;
            } else {
                result.push(paragraph);
                continue;
            }
        }

        result.push(paragraph);
    }

    result
}

fn calculate_paragraph_length_threshold(paragraphs_lengths: &[usize], percentile: f32) -> f32 {
    let mut unique_lengths: Vec<usize> = paragraphs_lengths.to_vec();
    unique_lengths.sort();
    unique_lengths.dedup();

    if unique_lengths.is_empty() {
        return 0.0; // Return 0 if there are no differences to calculate
    }

    // Sort the differences to calculate percentiles
    unique_lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Calculate the percentile index
    let index = ((percentile / 100.0) * (unique_lengths.len() - 1) as f32).round() as usize;
    let index = index.min(unique_lengths.len() - 1);

    unique_lengths[index] as f32
}

fn calculate_paragraph_gap_threshold(bbox: &[PdfPageText], percentile: f32) -> f32 {
    let mut unique_y_axis: Vec<f32> = bbox.iter().map(|segment| segment.bounding_box.y).collect();
    unique_y_axis.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_y_axis.dedup();

    // Calculate the differences between consecutive y-axis values
    let mut y_diffs = unique_y_axis
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect::<Vec<f32>>();

    if y_diffs.is_empty() {
        return 0.0; // Return 0 if there are no differences to calculate
    }

    // Sort the differences to calculate percentiles
    y_diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Calculate the percentile index
    let index = ((percentile / 100.0) * (y_diffs.len() - 1) as f32).round() as usize;
    let index = index.min(y_diffs.len() - 1);

    y_diffs[index]
}

#[cfg(test)]
mod tests {
    use crate::models::BoundingBox;

    use super::*;

    #[test]
    fn test_reduce_bbox_to_paragraphs() {
        let bboxes = vec![
            PdfPageText {
                page_index: 0,
                text: "Hello".into(),
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 20.0,
                },
            },
            PdfPageText {
                page_index: 0,
                text: "World".into(),
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 30.0,
                    width: 100.0,
                    height: 20.0,
                },
            },
        ];

        let paragraphs = reduce_bbox_to_paragraphs(&bboxes, 80.0);
        let reduced_paragraph = paragraphs.first().unwrap();
        assert_eq!(reduced_paragraph.text, "Hello World");
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(reduced_paragraph.bounding_box.x, 0.0);
        assert_eq!(reduced_paragraph.bounding_box.y, 0.0);
        assert_eq!(reduced_paragraph.bounding_box.width, 100.0);
        assert_eq!(reduced_paragraph.bounding_box.height, 50.0);
    }

    #[test]
    fn test_calculate_paragraph_threshold() {
        let bboxes = vec![
            PdfPageText {
                page_index: 0,
                text: "Line 1".into(),
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 20.0,
                },
            },
            PdfPageText {
                page_index: 0,
                text: "Line 2".into(),
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 25.0,
                    width: 100.0,
                    height: 20.0,
                },
            },
            PdfPageText {
                page_index: 0,
                text: "Line 3".into(),
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 60.0,
                    width: 100.0,
                    height: 20.0,
                },
            },
        ];

        let threshold = calculate_paragraph_gap_threshold(&bboxes, 1.0);
        assert_eq!(threshold, 25.0);
    }
}
