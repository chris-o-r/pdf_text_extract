use crate::models::PdfPageText;

pub fn reduce_bbox_to_paragraphs(bbox: &Vec<PdfPageText>, percentile: f32) -> Vec<PdfPageText> {
    let mut result: Vec<PdfPageText> = Vec::new();
    let mut current_paragraph: Option<PdfPageText> = None;
    let paragraph_threshold = calculate_paragraph_threshold(bbox, percentile);

    for segment in bbox.clone() {
        if current_paragraph.is_none() {
            current_paragraph = Some(segment);
        } else {
            let current_paragraph_y = current_paragraph.as_ref().unwrap().bounding_box.y;

            let y_diff = (segment.bounding_box.y
                - (current_paragraph_y + current_paragraph.as_ref().unwrap().bounding_box.height))
                .abs();

            if y_diff <= paragraph_threshold {
                current_paragraph = Some(current_paragraph.unwrap().combine(&segment));
            } else {
                result.push(current_paragraph.clone().unwrap());
                current_paragraph = Some(segment);
            }
        }
    }

    if current_paragraph.is_some() {
        result.push(current_paragraph.unwrap());
    }
    result
}

fn calculate_paragraph_threshold(bbox: &[PdfPageText], percentile: f32) -> f32 {
    let mut unique_y_axis: Vec<f32> = bbox.iter().map(|segment| segment.bounding_box.y).collect();
    unique_y_axis.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_y_axis.dedup();

    // Calculate the differences between consecutive y-axis values
    let mut y_diffs: Vec<f32> = unique_y_axis
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();

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

        let paragraphs = reduce_bbox_to_paragraphs(&bboxes, 85.0);
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

        let threshold = calculate_paragraph_threshold(&bboxes, 1.0);
        assert_eq!(threshold, 25.0);
    }
}
