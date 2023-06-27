use std::fmt::Write;
use std::path::Path;

use bitvec::prelude::*;
use image::{imageops::FilterType, GenericImageView, ImageError};

pub struct ImageHash<P: AsRef<Path>> {
    pub hash: BitVec,
    pub path: P,
}

impl<P: AsRef<Path>> ImageHash<P> {
    pub fn new(hash: BitVec, path: P) -> Self {
        Self { hash, path }
    }

    pub fn hexdigest(&self) -> String {
        let mut s = String::new();
        let mut byte = 0u8;
        let mut bit_count = 0;

        for bit in self.hash.iter() {
            byte = (byte << 1) | (*bit as u8);
            bit_count += 1;
            if bit_count == 4 {
                write!(&mut s, "{:01x}", byte).unwrap();
                byte = 0;
                bit_count = 0;
            }
        }

        // Handle leftover bits
        if bit_count != 0 {
            byte <<= 4 - bit_count; // Shift leftover bits to the left
            write!(&mut s, "{:01x}", byte).unwrap();
        }

        s
    }
}

// write a function to hash an image from its path
pub fn dhash_image<P: AsRef<Path>>(path: P, side_size: u32) -> Option<ImageHash<P>> {
    let mut img = image::open(path.as_ref())
        .map_err(|e| {
            if let ImageError::Decoding(_) = e {
                // if so, return None
                eprintln!("Error decoding {:?}: {:?}", path.as_ref().display(), e);
            }
            // users may pass in paths that are not images (especially in cases
            // when a directory is walked), which will result in
            // ImageError::Unsupported errors. But, since we don't want to impose
            // that a directory is all cleaned up, we won't print an error for
            // this case
        })
        .ok()?;

    // i did some benchmarking and found that grayscale-ing first is marginally
    // faster than resizing first
    img = img
        .grayscale()
        .resize_exact(side_size, side_size, FilterType::Triangle);

    let mut hash = BitVec::with_capacity((side_size * side_size) as usize);

    // compare each horizontal pixel pair (wrap for the last pixel)
    for row in 0..side_size {
        for col in 0..side_size {
            let left = img.get_pixel(col, row)[0];
            let right = img.get_pixel((col + 1) % side_size, row)[0];
            hash.push(left < right);
        }
    }
    Some(ImageHash::new(hash, path))
}
