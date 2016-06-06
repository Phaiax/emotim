
use image;
use image::{Pixel, Rgb, Rgba, GenericImage, DynamicImage};
use std::rc::Rc;

use std::path::{Path, PathBuf};
use std::char;




pub struct Emoticon {
    pub mean_color : Rgb<u8>,
    pub img: DynamicImage,
    pub unicode : char,
    pub unicode2 : Option<char>,
}

pub fn read_emoticons() -> Vec<Rc<Emoticon>> {
    let emotifolder = Path::new("assets/emoticons2");
    let mut emoticons = Vec::with_capacity(1700);
    for direntry in emotifolder.read_dir().unwrap().take(100) {
        let direntry = direntry.unwrap();
        emoticons.push(Rc::new(Emoticon::read_emoticon(direntry.path(),
                                                       direntry.file_name().into_string().unwrap())));
    }
    emoticons
}

pub fn mean<T>(img : &T) -> Rgb<u8>
    where T : GenericImage<Pixel = Rgba<u8>> {
    let mut mean = [0u32, 0, 0];
    let mut i = 0;
    for (_,_,pixel) in img.pixels() {
        let (r, g, b, a) = pixel.channels4();
        if a == 255 {
            mean[0] += r as u32;
            mean[1] += g as u32;
            mean[2] += b as u32;
            i += 1;
        }
    }
    mean[0] /= i;
    mean[1] /= i;
    mean[2] /= i;
    Rgb::from_channels(mean[0] as u8, mean[1] as u8, mean[2] as u8, 0)
}

impl Emoticon {

    fn str_to_unicode(s : &str) -> char {
        let unicodepoint = u32::from_str_radix(s, 16).expect(&format!("str {} is no hex string", s));
        let unicodepoint = char::from_u32(unicodepoint).unwrap();
        unicodepoint
    }

    fn read_emoticon(path : PathBuf, filename : String) -> Emoticon {
        let img = image::open(&path).unwrap();
        let mut ret = Emoticon {
            mean_color : mean(&img),
            img : img,
            unicode : 'c',
            unicode2 : None,
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

    pub fn distance(&self, to : Rgb<u8>) -> u32 {
        let (r1, g1, b1, _) = to.channels4();
        let (r2, g2, b2, _) = self.mean_color.channels4();
        ((r1 as i32 - r2 as i32).abs() +
        (g1 as i32 - g2 as i32).abs() +
        (b1 as i32 - b2 as i32).abs() ) as u32
    }
}
