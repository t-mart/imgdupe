mod hash;
mod path;

use std::collections::HashMap;

use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use crate::hash::dhash_image;
use crate::path::walk_files;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Group perceptually similar images from among image paths."
)]
struct Cli {
    /// Files to hash, or directories to walk to find files to hash. Only
    /// supported image formats will be hashed.
    ///
    /// A warning will be emitted if the extension of the file is one that
    /// should be supported, but was not able to be decoded.
    paths: Vec<String>,

    /// This parameter lets you adjust the precision of the similarity check.
    /// More precisely, it determines the dimension of one side of the square
    /// image after it has been resized. Lower values will result in loss of
    /// detail from the image (creating a more pixelated image, e.g., 8x8),
    /// whereas higher values will retain more detail (resulting in a higher
    /// resolution image, e.g., 64x64).
    ///
    /// Tune this value according to your desired false positive/negative rate.
    /// Note that computational complexity scales quadratically with it.
    #[arg(short, long, default_value_t = 8)]
    side: u32,
}

fn main() {
    let cli = Cli::parse();

    eprintln!("Hashing with {}x{} images ({}-bit hash)", cli.side, cli.side, cli.side * cli.side);

    let files = walk_files(cli.paths);
    eprintln!("Scanning {} files", files.len());

    // progress bar
    let style = indicatif::ProgressStyle::default_bar()
        .template("{spinner:.green} {elapsed_precise} of {duration_precise} [{bar:40.cyan/blue}] {percent}% ({human_pos:>7}/{human_len:7}) {per_sec:<11!}")
        .unwrap()
        .progress_chars("##-");

    // with rayon, hash all the images in parallel
    let hashes = files
        .par_iter()
        .progress_with_style(style)
        .filter_map(|path| dhash_image(path, cli.side))
        .collect::<Vec<_>>();

    // group the images by their hash
    let mut map = hashes.into_iter().fold(HashMap::new(), |mut map, hash| {
        map.entry(hash.hexdigest()).or_insert_with(Vec::new).push(hash.path);
        map
    });

    let total_images = map.values().map(|images| images.len()).sum::<usize>();
    eprintln!("Hashed {} images", total_images);

    // delete all entries with only one image
    map.retain(|_, images| images.len() > 1);

    let json = serde_json::to_string_pretty(&map).unwrap();
    println!("{}", json);
}
