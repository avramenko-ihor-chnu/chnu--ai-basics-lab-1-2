use image::{DynamicImage, ImageResult, Rgb, open};
use imageproc::{self, drawing::draw_line_segment_mut};
use std::collections::HashMap;

type DividedImage = Vec<DynamicImage>;

fn image_cutter(pic: &DynamicImage, number_of_cells_in_row: u32) -> ImageResult<Vec<DynamicImage>> {
    let rgb_image = pic.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    let cell_size = width / number_of_cells_in_row;
    let mut cells = Vec::new();

    for row in 0..number_of_cells_in_row {
        for col in 0..number_of_cells_in_row {
            let x = col * cell_size;
            let y = row * cell_size;

            let actual_width = cell_size.min(width - x);
            let actual_height = cell_size.min(height - y);

            let cell_image =
                image::imageops::crop_imm(&rgb_image, x, y, actual_width, actual_height);
            cells.push(DynamicImage::ImageRgb8(cell_image.to_image()));
        }
    }

    Ok(cells)
}

fn sugar(pic: &DynamicImage, number_of_cells_in_row: u32) -> ImageResult<DynamicImage> {
    let mut rgb_image = pic.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    let cell_size = width / number_of_cells_in_row;
    let red = Rgb([255u8, 0u8, 0u8]);

    for col in 1..number_of_cells_in_row {
        let x = col * cell_size;
        draw_line_segment_mut(
            &mut rgb_image,
            (x as f32, 0.0),
            (x as f32, height as f32),
            red,
        );
    }

    for row in 1..number_of_cells_in_row {
        let y = row * cell_size;
        draw_line_segment_mut(
            &mut rgb_image,
            (0.0, y as f32),
            (width as f32, y as f32),
            red,
        );
    }

    Ok(DynamicImage::ImageRgb8(rgb_image))
}

pub async fn maybe_load_reference() -> Result<HashMap<i32, DynamicImage>, image::ImageError> {
    (0..=9)
        .map(|i| {
            let path = format!("assets/{}.bmp", i);
            open(&path).map(|img| (i, img))
        })
        .collect()
}

pub fn event_vector_difference(lha: DividedImage, rha: DividedImage) -> ImageResult<u32> {
    let lha_vector = get_event_vector(lha)?;
    let rha_vector = get_event_vector(rha)?;
    lha_vector
        .iter()
        .zip(rha_vector.iter())
        .map(|(&l, &r)| Ok(l.abs_diff(r)))
        .sum()
}

fn count_black_pixels(pic: &DynamicImage) -> u32 {
    let rgb_image = pic.to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let mut count = 0;

    for x in 0..width {
        for y in 0..height {
            let pixel = rgb_image.get_pixel(x, y);
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
                count += 1;
            }
        }
    }
    count
}

pub fn get_event_vector(images: DividedImage) -> ImageResult<Vec<u32>> {
    Ok(images
        .iter()
        .map(|image| count_black_pixels(image))
        .collect())
}

pub struct TableRow {
    pub index_of_reference: usize,   // (0..=9)
    pub number_of_cells_in_row: u32, // (2..7)
    pub difference: u32,
}

pub fn guess_image(
    main_pic: &DynamicImage,
    reference: &HashMap<i32, DynamicImage>, // Changed from State<HashMap> to just HashMap
) -> Result<Vec<TableRow>, Box<dyn std::error::Error>> {
    let mut results: Vec<TableRow> = (3..7)
        .flat_map(|number_of_cells_in_row| {
            let divided_main_pic = image_cutter(main_pic, number_of_cells_in_row).unwrap();
            let pseudo_divided_main_pic = sugar(main_pic, number_of_cells_in_row).unwrap();
            let file_name = format!("sugar/{number_of_cells_in_row}.bmp");
            pseudo_divided_main_pic.save(file_name).unwrap();

            let references: Vec<&DynamicImage> = (0..=9).map(|i| &reference[&i]).collect();

            (0..=9).map(move |index_of_reference| {
                let item_of_reference = references[index_of_reference];
                let divided_reference =
                    image_cutter(item_of_reference, number_of_cells_in_row).unwrap();

                let difference =
                    event_vector_difference(divided_main_pic.clone(), divided_reference).unwrap();

                TableRow {
                    index_of_reference,
                    number_of_cells_in_row,
                    difference,
                }
            })
        })
        .collect::<Vec<_>>();

    results.sort_by_key(|row| row.difference);
    Ok(results)
}
