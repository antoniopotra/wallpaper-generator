use chrono::{Datelike, Local, Timelike};
use clap::Parser;
use opencv::{
    core::{CV_8UC3, Mat, MatTraitConst, Rect, Scalar, Size, Vector},
    imgcodecs::{IMREAD_COLOR, imread, imwrite},
    imgproc::{INTER_AREA, resize},
};
use rand::{rng, seq::SliceRandom};
use std::{fs::read_dir, path::PathBuf};
use wallpaper::set_from_path;

const IMAGE_WIDTH: i32 = 480;
const IMAGE_HEIGHT: i32 = 720;
const IMAGES_PER_ROW_COUNT: i32 = 16;
const IMAGES_PER_COLUMN_COUNT: i32 = 6;

#[derive(Parser, Debug)]
#[command(version)]
struct Arguments {
    /// Path of the input folder
    input_path: String,

    /// Path of the output folder
    output_path: String,
}

fn main() {
    let arguments = Arguments::parse();

    let Ok(directory) = read_dir(&arguments.input_path) else {
        println!("Failed to open directory at: {}.", { arguments.input_path });
        return;
    };

    let mut images: Vec<Mat> = directory
        .filter_map(|entry| {
            let Ok(entry) = entry else {
                return None;
            };

            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let image = imread(path.to_str().unwrap(), IMREAD_COLOR).unwrap();
            if image.empty() {
                return None;
            }

            let mut resized_image = Mat::default();
            resize(
                &image,
                &mut resized_image,
                Size::new(IMAGE_WIDTH, IMAGE_HEIGHT),
                0.0,
                0.0,
                INTER_AREA,
            )
            .unwrap();

            Some(resized_image)
        })
        .collect();

    let expected_images_count = (IMAGES_PER_ROW_COUNT * IMAGES_PER_COLUMN_COUNT) as usize;
    if images.len() < expected_images_count {
        println!(
            "Found {} images in the {} directory, expected {}.",
            images.len(),
            arguments.input_path,
            expected_images_count
        );
        return;
    }

    images.shuffle(&mut rng());
    let images = images[0..expected_images_count].to_vec();

    let wallpaper_width = IMAGE_WIDTH * IMAGES_PER_ROW_COUNT;
    let wallpaper_height = IMAGE_HEIGHT * IMAGES_PER_COLUMN_COUNT;

    let mut wallpaper = Mat::new_rows_cols_with_default(
        wallpaper_height,
        wallpaper_width,
        CV_8UC3,
        Scalar::new(0.0, 0.0, 0.0, 0.0),
    )
    .unwrap();

    for (index, image) in images.iter().enumerate() {
        let row = index as i32 / IMAGES_PER_ROW_COUNT;
        let column = index as i32 % IMAGES_PER_ROW_COUNT;

        let x = column * IMAGE_WIDTH;
        let y = row * IMAGE_HEIGHT;

        let mut region_of_interest =
            Mat::roi_mut(&mut wallpaper, Rect::new(x, y, IMAGE_WIDTH, IMAGE_HEIGHT)).unwrap();
        let _ = image.copy_to(&mut region_of_interest);
    }

    let now = Local::now();
    let wallpaper_name = format!(
        "wallpaper-{:02}-{:02}-{:04}-{:02}-{:02}-{:02}.png",
        now.day(),
        now.month(),
        now.year(),
        now.hour(),
        now.minute(),
        now.second(),
    );

    let mut wallpaper_path = PathBuf::from(&arguments.output_path);
    wallpaper_path.push(&wallpaper_name);
    let wallpaper_path = wallpaper_path.to_str().unwrap();

    let _ = imwrite(wallpaper_path, &wallpaper, &Vector::new());
    println!("Wallpaper saved to {wallpaper_path}.");

    let _ = set_from_path(wallpaper_path);
}
