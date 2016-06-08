#![feature(test)]

extern crate image;
extern crate test;

mod emoticons;
mod hsv;

use image::{GenericImage, DynamicImage};
use std::path::{Path};
use std::rc::Rc;


fn read_input_image() -> DynamicImage {
    let inputimagepath = Path::new("assets/input/Munch_Schrei_6.jpg");
    let img = image::open(&inputimagepath).unwrap();
    img
}

pub struct Emoimage {
    pub width : u32,
    pub height : u32,
    pub emopixels : Vec<Rc<emoticons::Emoticon>>,
}

fn find_best_smileys(img : &mut DynamicImage, frac : u32, emos : &Vec<Rc<emoticons::Emoticon>>) -> Emoimage {
    let height = img.height() / frac;
    let width = img.width() / frac;
    //let pxheight = img.height();
    //let pxwidth = img.width();
    let mut out = Vec::with_capacity(width as usize * height as usize);
    //let raw = img.raw_pixels();



    for h in 0..height {
        for w in 0..width {
            println!("{} {}", h, w);

            let subimg = img.sub_image(w * frac, h * frac, frac, frac);
            let subimghsv = hsv::HsvImage::from_image(&subimg);
            let subimghist = subimghsv.reduce_dynamic().histogram();

            let mut closest = None;
            let mut similarity = 0.0;
            for e in emos {
                let esim = e.hist.similarity2(&subimghist);
                if esim > similarity {
                    closest = Some(e.clone());
                    similarity = esim;
                }
            }
            //let e = all[i as usize % all.len()].clone();
            out.push(closest.unwrap());

        }
    }
    Emoimage {
        width : width,
        height : height,
        emopixels : out
    }
}

impl Emoimage {

    fn print(&self) {

        for line in self.emopixels.chunks(self.width as usize) {
            for s in line {
                if let Some(u2) = s.unicode2 {
                    print!("{}", u2);
                }
                print!("{}", s.unicode);
            }
            print!("\n");
        }
    }
}

fn main()  {
    println!("Hey");
    let emos = emoticons::read_emoticons();

    let mut ii = read_input_image();

    let emoimg = find_best_smileys(&mut ii, 20, &emos);
    emoimg.print();


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
