
use image;
use image::{DynamicImage};
use std::rc::Rc;

use std::path::{Path, PathBuf};
use std::char;
use std::fs::File;

use hsv;

pub struct Emoticon {
    pub img: DynamicImage,
    pub unicode : char,
    pub unicode2 : Option<char>,
    pub hsv : hsv::HsvImage,
    pub hist : hsv::ReducedHsvHistogram,
}

pub fn read_emoticons() -> Vec<Rc<Emoticon>> {
    let emotifolder = Path::new("assets/emoticons2");
    let mut emoticons = Vec::with_capacity(1700);
    for (i, direntry) in emotifolder.read_dir().unwrap().enumerate() {
        let direntry = direntry.unwrap();
        emoticons.push(Rc::new(Emoticon::read_emoticon(direntry.path(),
                                                       direntry.file_name().into_string().unwrap())));
        println!("{}", i);
    }
    println!("");
    emoticons
}


impl Emoticon {

    fn str_to_unicode(s : &str) -> char {
        let unicodepoint = u32::from_str_radix(s, 16).expect(&format!("str {} is no hex string", s));
        let unicodepoint = char::from_u32(unicodepoint).unwrap();
        unicodepoint
    }

    fn read_emoticon(path : PathBuf, filename : String) -> Emoticon {
        let img = image::open(&path).unwrap();
        //let blurred = img.blur(6.0);


        let hsv = hsv::HsvImage::from_image(&img);
        let hsvreduced = hsv.reduce_dynamic();
        let hist = hsvreduced.histogram();

        let ref mut fout = File::create(&Path::new(&format!("assets/blurred/{}", filename))).unwrap();
        let _ = hsvreduced.to_rgba().save(fout, image::PNG).unwrap();

        let mut ret = Emoticon {
            img : img,
            unicode : 'c',
            unicode2 : None,
            hsv : hsv,
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

}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::path::PathBuf;
    use std::rc::Rc;


    fn open_emoticon() -> Emoticon {
        let inputimagepath = PathBuf::from("assets/emoticons2/00a9.png");
        let img = Emoticon::read_emoticon(inputimagepath, "00a9.png".to_string());
        img
    }

    #[bench]
    fn bench_open_emoticon(b: &mut Bencher) {
        b.iter(|| open_emoticon());
    }

    fn open_emoticon_rc() -> Rc<Emoticon> {
        let inputimagepath = PathBuf::from("assets/emoticons2/00a9.png");
        Rc::new(Emoticon::read_emoticon(inputimagepath,
                                        "00a9.png".to_string()))
    }

    #[bench]
    fn bench_open_emoticon_rc(b: &mut Bencher) {
        b.iter(|| open_emoticon_rc());
    }
}


