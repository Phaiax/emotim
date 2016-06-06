extern crate image;

mod emoticons;

use image::{Pixel, Rgb, GenericImage, DynamicImage};
use std::path::{Path};
use std::rc::Rc;
fn read_input_image() -> DynamicImage {
    let inputimagepath = Path::new("assets/input/Munch_Schrei_6.jpg");
    let img = image::open(&inputimagepath).unwrap();
    img
}

struct Emoimage {
    width : u32,
    height : u32,
    emopixels : Vec<Rc<emoticons::Emoticon>>,
}

fn find_best_smileys(img : &mut DynamicImage, frac : u32, emos : &Vec<Rc<emoticons::Emoticon>>) -> Emoimage {
    let height = img.height() / frac;
    let width = img.width() / frac;
    let pxheight = img.height();
    let pxwidth = img.width();
    let mut out = Vec::with_capacity(width as usize * height as usize);
    let raw = img.raw_pixels();


    for w in 0..width {
        for h in 0..height {

            let subimg = img.sub_image(w * frac, h * frac, frac, frac);
            let meansubimg = emoticons::mean(&subimg);

            let mut closest = None;
            let mut distance = 1000;
            for e in emos {
                let edist = e.distance(meansubimg);
                if edist < distance {
                    closest = Some(e.clone());
                    distance = edist;
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
