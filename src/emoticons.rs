//! Read and prepare the emoticon pixels

use image;
use image::{DynamicImage};
use std::rc::Rc;

use std::path::PathBuf;
use std::char;
use std::fs::File;
use std::fmt;
use std::io;
use std::io::Write;

use hsl;



/// A list of `Emoticon`s.
pub type Emoticons = Vec<Rc<Emoticon>>;

/// Reads all emoticons from assets/emoticons2.
///
/// Expects the filename to be `[<unicodepoint>-]<unicodepoint>.png`
/// where `<unicodepoint>` is a hex number. Eg: `0023-20e3.png` or `1f004.png`
pub fn read_emoticons() -> Emoticons {
    let emotifolder = PathBuf::from("assets/emoticons2".to_string());
    let mut emoticons = Vec::with_capacity(1700);
    println!("Read folder {}:", emotifolder.display());
    for (i, direntry) in emotifolder.read_dir()
                                    .expect("Folder not found")
                                    .enumerate() {
        if let Ok(direntry) = direntry {
            if let Ok(filetype) = direntry.file_type() {
                if ! filetype.is_file() {
                    continue;
                }
            }
            emoticons.push(Rc::new(Emoticon::read_emoticon(direntry.path())));
            // progress
            print!("\r{}", i);
            io::stdout().flush().ok();
        }
    }
    println!("");
    emoticons
}

/// An emoticon with metadata like histogram and unicode representation.
pub struct Emoticon {
    pub img: DynamicImage,
    pub unicode : char,
    pub unicode2 : Option<char>,
    pub filename : String,
    pub hsl : hsl::HslImage,
    pub hslreduced : hsl::HslImageWithReducedDepth,
    pub hist : hsl::HslHistogram,
}

impl Emoticon {

    /// Reads emoticon from png file.
    ///
    /// Expects the filename to be `[<unicodepoint>-]<unicodepoint>.png`
    /// where `<unicodepoint>` is a hex number. Eg: `0023-20e3.png` or `1f004.png`
    pub fn read_emoticon(path : PathBuf) -> Emoticon {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let img = image::open(&path).unwrap();
        let hsl = hsl::HslImage::from_image(&img);
        let hslreduced = hsl.reduce_dynamic();
        let hist = hslreduced.histogram();

        let mut ret = Emoticon {
            img : img,
            unicode : ' ',
            unicode2 : None,
            filename : filename.to_string(),
            hsl : hsl,
            hslreduced : hslreduced,
            hist : hist,
        };

        if filename.contains("-") {
            let mut split = filename.split(|c| c == '-' || c == '.');
            ret.unicode2 = Some(Emoticon::str_to_unicode(split.next().unwrap()));
            ret.unicode = Emoticon::str_to_unicode(split.next().unwrap());
        } else {
            ret.unicode = Emoticon::str_to_unicode(&filename[..filename.len() - 4]);
        }
        ret
    }

    /// Converts a hex number representation of a unicodepoint like `20e3` to a `char`
    fn str_to_unicode(s : &str) -> char {
        let unicodepoint = u32::from_str_radix(s, 16).expect(&format!("str {} is no hex string", s));
        char::from_u32(unicodepoint).expect(&format!("str {} does not represent a valid unicodepoint", s))
    }

    /// For debugging purposes, save reduced hsl image (convert back to rgb first) into out/reduced
    pub fn save_reduced(&self) {
        let mut path = PathBuf::new();
        path.push("out/reduced");
        path.push(&self.filename);
        let ref mut fout = File::create(path).unwrap();
        let _ = self.hslreduced.extend_dynamic().to_rgba().save(fout, image::PNG).unwrap();
    }
}

impl fmt::Display for Emoticon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(u2) = self.unicode2 {
            write!(f, "{}", u2).unwrap()
        }
        write!(f, "{}", self.unicode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::path::PathBuf;
    use std::rc::Rc;


    fn open_emoticon() -> Emoticon {
        let inputimagepath = PathBuf::from("assets/emoticons2/00a9.png");
        let img = Emoticon::read_emoticon(inputimagepath);
        img
    }

    #[bench]
    fn bench_open_emoticon(b: &mut Bencher) {
        b.iter(|| open_emoticon());
    }

    fn open_emoticon_rc() -> Rc<Emoticon> {
        let inputimagepath = PathBuf::from("assets/emoticons2/00a9.png");
        Rc::new(Emoticon::read_emoticon(inputimagepath))
    }

    #[bench]
    fn bench_open_emoticon_rc(b: &mut Bencher) {
        b.iter(|| open_emoticon_rc());
    }
}


