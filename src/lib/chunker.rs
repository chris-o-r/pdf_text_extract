use crate::pdf::{BoundingBox, PdfPageText};

pub fn reduce_bbox_to_paragraphs(bbox: &Vec<PdfPageText>) -> Vec<PdfPageText> {
    let mut result: Vec<PdfPageText> = Vec::new();
    let mut current_paragraph: Option<PdfPageText> = None;
    let paragraph_threshold = calculate_paragraph_threshold(bbox, 1.0);

    for segment in bbox.clone() {
        if current_paragraph.is_none() {
            current_paragraph = Some(segment);
        } else {
            let current_paragraph_y = current_paragraph.as_ref().unwrap().bounding_box.y;

            if segment.bounding_box.y < current_paragraph_y {
                result.push(current_paragraph.unwrap());
                current_paragraph = Some(segment);
            } else {
                let y_diff = (segment.bounding_box.y
                    - (current_paragraph_y
                        + current_paragraph.as_ref().unwrap().bounding_box.height))
                    .abs();

                if y_diff <= paragraph_threshold {
                    current_paragraph =
                        Some(combine_bounding_boxes(current_paragraph.unwrap(), segment));
                } else {
                    result.push(current_paragraph.unwrap());
                    current_paragraph = None
                }
            }
        }
    }

    if current_paragraph.is_some() {
        result.push(current_paragraph.unwrap());
    }
    result
}

fn combine_bounding_boxes(bbox_1: PdfPageText, bbox_2: PdfPageText) -> PdfPageText {
    PdfPageText {
        page_index: bbox_1.page_index,
        text: format!("{} {}", bbox_1.text, bbox_2.text),
        bounding_box: BoundingBox {
            x: bbox_1.bounding_box.x.min(bbox_2.bounding_box.x),
            y: bbox_1.bounding_box.y.min(bbox_2.bounding_box.y),
            width: (bbox_1.bounding_box.x + bbox_1.bounding_box.width)
                .max(bbox_2.bounding_box.x + bbox_2.bounding_box.width)
                - bbox_1.bounding_box.x.min(bbox_2.bounding_box.x),
            height: (bbox_1.bounding_box.y + bbox_1.bounding_box.height)
                .max(bbox_2.bounding_box.y + bbox_2.bounding_box.height)
                - bbox_1.bounding_box.y.min(bbox_2.bounding_box.y),
        },
    }
}

fn calculate_paragraph_threshold(bbox: &Vec<PdfPageText>, standard_deviation: f32) -> f32 {
    let mut unique_y_axis: Vec<f32> = bbox.iter().map(|segment| segment.bounding_box.y).collect();
    unique_y_axis.sort_by(|a, b| a.partial_cmp(b).unwrap());
    unique_y_axis.dedup();

    // Calculate the differences between consecutive y-axis values
    let y_diffs: Vec<f32> = unique_y_axis
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();

    if y_diffs.is_empty() {
        return 0.0; // Return 0 if there are no differences to calculate
    }

    // Get the standard deviation of the y-axis differences
    let mean_diff = y_diffs.iter().sum::<f32>() / y_diffs.len() as f32;
    let std_dev_diff = (y_diffs
        .iter()
        .map(|diff| (diff - mean_diff).powi(2))
        .sum::<f32>()
        / y_diffs.len() as f32)
        .sqrt();

    // Return the mean difference plus standard deviation multiplier
    mean_diff + standard_deviation * std_dev_diff
}
