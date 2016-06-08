//! Converts normal images into emoticon versions by replacing chunks of the
//! original image with emoticons of similar color.
//!
//! ```
//!     use std::path::Path;
//!     let emos = read_emoticons();
//!     let mut ii = read_input_image("Munch_Schrei_6.jpg");
//!     let emoimg = Emoimage::new(&mut ii, 20, &emos, ComparisationMethod::Maxima);
//!     println!("{}", emoimg);
//!     emoimg.save(&Path::new("out/munch_max.png"));
//! ```
//!
//! This crate is much much faster in release mode.
//!
//! Since some paths are hardcoded, you need to have `assets/emoticons2/*` in the working directory.

#![feature(test)]

extern crate image;
extern crate test;

pub mod emoticons;
pub mod hsl;

use image::{GenericImage, DynamicImage, RgbaImage};

use std::path::{PathBuf, Path};
use std::fs::File;
use std::rc::Rc;
use std::fmt;
use std::io;
use std::io::Write;

use emoticons::Emoticons;
pub use emoticons::read_emoticons;

/// Reads a normal image from `assets/input/<filename>`.
pub fn read_input_image(filename : &str) -> DynamicImage {
    let mut inputimagepath = PathBuf::new();
    inputimagepath.push("assets/input");
    inputimagepath.push(filename);
    image::open(&inputimagepath).expect(&format!("image {} not found", inputimagepath.display()))
}

/// An image made out of emoticons
pub struct Emoimage {
    pub width : u32,
    pub height : u32,
    pub emopixels : Vec<Rc<emoticons::Emoticon>>,
}

/// Different methods to calculate the corresponding emoticons.
pub enum ComparisationMethod {
    Correlation,
    Maxima
}

impl Emoimage {
    /// Does the calculation.
    pub fn new(img : &mut DynamicImage,
               frac : u32,
               emoticons : &Emoticons,
               method : ComparisationMethod) -> Emoimage {

        let height = img.height() / frac;
        let width = img.width() / frac;
        let mut pixels = Vec::with_capacity(width as usize * height as usize);

        println!("Finding best emoticon for chunk of input image:");
        for h in 0..height {
            for w in 0..width {
                // progress
                print!("\r Chunk @ h:{} w:{}", h, w);
                io::stdout().flush().ok();

                let subimg = img.sub_image(w * frac, h * frac, frac, frac);
                let subimghsv = hsl::HslImage::from_image(&subimg);
                let subimghist = subimghsv.reduce_dynamic().histogram();

                let mut the_chosen_one = None;
                let mut highest_similarity = 0.0;
                for e in emoticons {
                    let similarity = match method {
                        ComparisationMethod::Correlation => e.hist.similarity_by_correlation(&subimghist),
                        ComparisationMethod::Maxima => e.hist.similarity_by_maxima(&subimghist),
                    };
                    if similarity > highest_similarity {
                        the_chosen_one = Some(e.clone());
                        highest_similarity = similarity;
                    }
                }
                pixels.push(the_chosen_one.unwrap());
            }
        }
        println!("\r Done.");
        Emoimage {
            width : width,
            height : height,
            emopixels : pixels
        }
    }

    /// Saves the calculated emoticons as image
    pub fn save(&self, path : &Path) {
        // Calculate dimensions
        // Use first emoticon as base for height / width
        let exampleemo = self.emopixels.first().unwrap();
        let height = exampleemo.img.height() * self.height;
        let width = exampleemo.img.width() * self.width;
        let raw = vec![0 ; (height * width * 4) as usize];
        let img = RgbaImage::from_raw(width, height, raw).unwrap();
        let mut img = DynamicImage::ImageRgba8(img);
        for h in 0..self.height {
            for w in 0..self.width {
                img.copy_from(&self.emopixels[(h * self.width + w) as usize].img,
                              w * exampleemo.img.width(),
                              h * exampleemo.img.height());
            }
        }
        let ref mut fout = File::create(path).unwrap();
        let _ = img.save(fout, image::PNG).unwrap();
    }
}

impl fmt::Display for Emoimage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.emopixels.chunks(self.width as usize) {
            for s in line {
                try!(write!(f, "{}", s));
            }
            try!(write!(f, "\n"));
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    //use super::*;
    use test::Bencher;
    use image::{DynamicImage};
    use image;
    use std::path::Path;


    fn open_image() -> DynamicImage {
        let inputimagepath = Path::new("assets/emoticons2/00a9.png");
        let img = image::open(&inputimagepath).unwrap();
        img
    }

    #[bench]
    fn bench_open_image(b: &mut Bencher) {
        b.iter(|| open_image());
    }
}
