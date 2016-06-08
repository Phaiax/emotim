
use image::{Pixel, Rgba, GenericImage, DynamicImage, RgbaImage};
use std::f32;
#[derive(Debug,Clone)]
pub struct Hsv {
    /// Skip hexagon to circle transformation and use simplified conversion
    /// alpha = 0.5 * (2 R - G - B)
    /// beta = sqrt(3) / 2 * (G - B)
    /// h2 = atan(beta, alpha)
    pub h2 : u8,
    /// c2 = sqrt (alpha^2 * beta^2)
    pub c2 : u8,
    /// Lightness
    /// l = 0.5 * (m + M)
    pub l : u8,
    /// Copy alpha from rgba
    pub a : u8,
}

impl Hsv {
    fn new(h2 : u8, c2 : u8, l : u8, a : u8) -> Hsv {
        Hsv {h2 : h2, c2 : c2, l : l , a : a }
    }

    fn distance(&self, other : &Hsv) -> u32 {
        ((self.h2 as i32 - other.h2 as i32).abs() +
        (self.c2 as i32 - other.c2 as i32).abs() +
        (self.l as i32 - other.l as i32).abs() +
        (self.a as i32 - other.a as i32).abs() ) as u32
    }

    pub fn to_rgba(&self) -> Rgba<u8> {
        let h_tick = self.h2 as f32 / (256.0/6.0);
        let x = self.c2 as f32 * ( 1.0 - (h_tick % 2.0  - 1.0).abs());
        let (mut r1, mut g1, mut b1) = if 0.0 <= h_tick && h_tick < 1.0 { ( self.c2 as f32, x, 0.0) }
                      else if 1.0 <= h_tick && h_tick < 2.0 { ( x, self.c2 as f32, 0.0) }
                      else if 2.0 <= h_tick && h_tick < 3.0 { ( 0.0, self.c2 as f32, x) }
                      else if 3.0 <= h_tick && h_tick < 4.0 { ( 0.0, x, self.c2 as f32) }
                      else if 4.0 <= h_tick && h_tick < 5.0 { ( x, 0.0, self.c2 as f32) }
                      else if 5.0 <= h_tick && h_tick <= 6.0 { ( self.c2 as f32, 0.0, x) }
                      else { (0.0, 0.0, 0.0) };
        let m = self.l as f32 - 0.5 * self.c2 as f32;
        if m < 0. {
            if r1 < -m { r1 = -m };
            if g1 < -m { g1 = -m };
            if b1 < -m { b1 = -m };
        }
        if m > 0. {
            if r1 + m > 255. { r1 = 255.-m };
            if g1 + m > 255. { g1 = 255.-m };
            if b1 + m > 255. { b1 = 255.-m };
        }
        Rgba::from_channels((r1+m) as u8, (g1+m) as u8, (b1+m) as u8, self.a)
    }

    pub fn extend_dynamic(&self) -> Hsv {
        Hsv { h2 : self.h2 * 16,
              c2 : self.c2 * 16,
              l : self.l * 16,
              a : self.a * 255}
    }

    pub fn reduce_dynamic(&self) -> Hsv {
        Hsv {
            h2 : self.h2 / 16,
            c2 : self.c2 / 16,
            l : self.l / 16,
            a : if self.a > 204 { 1 } else { 0 },
        }
    }

}

pub struct HsvImage {
    pub pixels : Vec<Hsv>,
    pub height : u32,
    pub width : u32,
}


impl HsvImage {
    pub fn from_image<T>(img : &T) -> HsvImage
        where T : GenericImage<Pixel = Rgba<u8>> {
        let mut hsv = Vec::with_capacity((img.width() * img.height()) as usize );

        for (_,_,pixel) in img.pixels() {
            let (r, g, b, a) = pixel.channels4();
            let r = r as f32;
            let g = g as f32;
            let b = b as f32;
            let alpha : f32 = 0.5f32 * ( 2f32 * r - g - b);
            let beta : f32 = 3f32.sqrt() / 2f32 * (g - b);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            hsv.push( Hsv {
                h2 : (beta.atan2(alpha) * 128.0 / f32::consts::PI) as u8,
                c2 : ( (alpha.powi(2) + beta.powi(2)).sqrt()  ) as u8,
                l : ((max + min) / 2.0) as u8,
                a : a,
            });
        }

        HsvImage {
            pixels : hsv,
            height : img.height(),
            width : img.width(),
        }
    }

    pub fn to_rgba(&self) -> DynamicImage {
        let mut raw = Vec::with_capacity(self.width as usize * self.height as usize * 4);
        for p in &self.pixels {
            let rgba = p.to_rgba();
            raw.push(rgba.data[0]);
            raw.push(rgba.data[1]);
            raw.push(rgba.data[2]);
            raw.push(rgba.data[3]);
        }

        DynamicImage::ImageRgba8(RgbaImage::from_raw(self.width,
                                                     self.height,
                                                     raw).unwrap())
    }

    /// alpha : 2 values: On (>80%), Off (<80%)
    /// l : 16 steps
    /// h2: 16 steps
    /// c2: 16 steps
    /// Makes a total of 16^3=4096 colors
    pub fn reduce_dynamic(&self) -> ReducedHsvImage {
        let mut hsv = Vec::with_capacity(self.pixels.len());

        for h in &self.pixels {
            hsv.push( h.reduce_dynamic() );
        }

        ReducedHsvImage(HsvImage {
            pixels : hsv,
            height : self.height,
            width : self.width,
        })
    }

    pub fn reduce_dynamic_self(&mut self) {
        for h in &mut self.pixels {
            h.h2 = h.h2 / 16;
            h.h2 = h.h2 * 16;
            h.c2 = if h.c2 < 16 { (h.c2 + 12) / 16 } else { h.c2 / 16 } ;
            h.c2 = h.c2 * 16;
            h.l = if h.l < 16 { (h.l + 12) / 16 } else { h.l / 16 } ;
            h.l = h.l * 16;
            h.a = if h.a > 204 { 255 } else {0};
        }
    }

    pub fn get(&self, x : u32, y : u32) -> Hsv {
        self.pixels[(y*self.width + x ) as usize ].clone()
    }

}

pub struct ReducedHsvImage (HsvImage);

pub struct ReducedHsvHistogram {
    /// [h2][c2][l]
    pub distribution : [[[u32 ; 16] ; 16] ; 16],
    pub maxima : Vec<(Hsv, f32)>,
}


impl ReducedHsvImage {
    pub fn histogram(&self) -> ReducedHsvHistogram {
        ReducedHsvHistogram::from_reduced_hsv_image(&self)
    }
    pub fn to_rgba(&self) -> DynamicImage {
        let mut raw = Vec::with_capacity(self.0.width as usize * self.0.height as usize * 4);
        for p in &self.0.pixels {
            let rgba = p.extend_dynamic().to_rgba();
            raw.push(rgba.data[0]);
            raw.push(rgba.data[1]);
            raw.push(rgba.data[2]);
            raw.push(rgba.data[3]);
        }

        DynamicImage::ImageRgba8(RgbaImage::from_raw(self.0.width,
                                                     self.0.height,
                                                     raw).unwrap())
    }
}

impl ReducedHsvHistogram {
    fn from_reduced_hsv_image(img : &ReducedHsvImage) -> ReducedHsvHistogram {
        let mut ret = ReducedHsvHistogram {
            distribution : [[[0 ; 16] ; 16] ; 16],
            maxima : Vec::with_capacity(5),
        };
        for h in &img.0.pixels {
            if h.a == 0 {
                continue;
            }
            ret.distribution[h.h2 as usize][h.c2 as usize][h.l as usize] += 1;
        }
        ret.find_maxima();
        ret.smooth()
        //ret
    }

    fn find_maxima(&mut self) {
        let smoothed = self.smooth();
        // Strategy:
        // After finding maximas, calculate the size of the corresponding maxima
        // by adding up all direct neighbours values
        // store the maximas

        for ih in 1..15 {
            for ic in 1..15 {
                for il in 1..15 {
                    let center = smoothed.distribution[ih][ic][il];
                    if center == 0 { continue; }
                    let found_greater =
                        // top (ih += 1)
                             if smoothed.distribution[ih+1][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if smoothed.distribution[ih+1][ic-1][il+0] > center { true }
                        else if smoothed.distribution[ih+1][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih+1][ic+0][il-1] > center { true } // mid ( ic += 0)  // front (il -=1)
                        else if smoothed.distribution[ih+1][ic+0][il+0] > center { true }
                        else if smoothed.distribution[ih+1][ic+0][il+1] > center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih+1][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if smoothed.distribution[ih+1][ic+1][il+0] > center { true }
                        else if smoothed.distribution[ih+1][ic+1][il+1] > center { true }                    // back (il += 1)

                        // mid (ih += 0)
                        else if smoothed.distribution[ih+0][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if smoothed.distribution[ih+0][ic-1][il+0] > center { true }
                        else if smoothed.distribution[ih+0][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih+0][ic+0][il-1] > center { true } // mid ( ic += 0) // front (il -=1)
                        //else if smoothed.distribution[ih+0][ic+0][il+0] > center { true }
                        else if smoothed.distribution[ih+0][ic+0][il+1] >= center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih+0][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if smoothed.distribution[ih+0][ic+1][il+0] >= center { true }
                        else if smoothed.distribution[ih+0][ic+1][il+1] >= center { true }                    // back (il += 1)

                        // bot (ih -= 1)
                        else if smoothed.distribution[ih-1][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if smoothed.distribution[ih-1][ic-1][il+0] > center { true }
                        else if smoothed.distribution[ih-1][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih-1][ic+0][il-1] > center { true } // mid ( ic += 0) // front (il -=1)
                        else if smoothed.distribution[ih-1][ic+0][il+0] >= center { true }
                        else if smoothed.distribution[ih-1][ic+0][il+1] >= center { true }                    // back (il += 1)
                        else if smoothed.distribution[ih-1][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if smoothed.distribution[ih-1][ic+1][il+0] >= center { true }
                        else if smoothed.distribution[ih-1][ic+1][il+1] >= center { true }                    // back (il += 1)
                        else { false } ;
                    if ! found_greater {
                        let sum =
                            // top (ih += 1)
                            smoothed.distribution[ih+1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            smoothed.distribution[ih+1][ic-1][il+0] +
                            smoothed.distribution[ih+1][ic-1][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih+1][ic+0][il-1] + // mid ( ic += 0)  // front (il -=1)
                            smoothed.distribution[ih+1][ic+0][il+0] +
                            smoothed.distribution[ih+1][ic+0][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih+1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            smoothed.distribution[ih+1][ic+1][il+0] +
                            smoothed.distribution[ih+1][ic+1][il+1] +                    // back (il += 1)

                            // mid (ih += 0)
                            smoothed.distribution[ih+0][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            smoothed.distribution[ih+0][ic-1][il+0] +
                            smoothed.distribution[ih+0][ic-1][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih+0][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                            smoothed.distribution[ih+0][ic+0][il+0] +
                            smoothed.distribution[ih+0][ic+0][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih+0][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            smoothed.distribution[ih+0][ic+1][il+0] +
                            smoothed.distribution[ih+0][ic+1][il+1] +                    // back (il += 1)

                            // bot (ih -= 1)
                            smoothed.distribution[ih-1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            smoothed.distribution[ih-1][ic-1][il+0] +
                            smoothed.distribution[ih-1][ic-1][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih-1][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                            smoothed.distribution[ih-1][ic+0][il+0] +
                            smoothed.distribution[ih-1][ic+0][il+1] +                    // back (il += 1)
                            smoothed.distribution[ih-1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            smoothed.distribution[ih-1][ic+1][il+0] +
                            smoothed.distribution[ih-1][ic+1][il+1] ;                    // back (il += 1)
                        self.maxima.push((Hsv{
                                                h2 : ih as u8, c2 : ic as u8, l : il as u8, a : 1
                                          },
                                          sum as f32 / 64. )); // 64 is the sum of the smooth factors for all neighbours
                    }

                }
            }
        }
        //println!("Unsorted {:?}", self.maxima);
        self.sort_maxima();
        //println!("Sorted {:?}", self.maxima);
    }

    // smallest first
    fn sort_maxima(&mut self) {
        self.maxima.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        while self.maxima.len() > 5 || (self.maxima.len() >= 1 && self.maxima.first().unwrap().1 < 1. ) {
            self.maxima.remove(0);
        }
    }


    pub fn similarity(&self, other : &ReducedHsvHistogram) -> f32 {
        let mut distance = 0.0;
        // compare each with every maxima, multiply by distance and max(max)
        for mymax in &self.maxima {
            for othermax in &other.maxima {
                let mut d = 5.0 / (mymax.0.distance(&othermax.0) as f32);
                d *= mymax.1 * othermax.1 / 2.0f32;
                distance += d;
            }
        }
        distance
    }

    pub fn similarity2(&self, other : &ReducedHsvHistogram) -> f32 {
        let mut correlation = 0.0;
        for ih in 0..16 {
            for ic in 0..16 {
                for il in 0..16 {
                    correlation += self.distribution[ih][ic][il] as f32 *
                                   other.distribution[ih][ic][il] as f32;
                }
            }
        }
        correlation
    }

    /// Smooth via gaussian kernel
    ///            1-----2------1
    ///       2    | 4     2    |
    ///  1------2------1        |
    ///  |         |   |        |
    ///  |         2   | 4      2
    ///  |    4    | 8 |    4   |
    ///  2      4  |   2        |
    ///  |         |   |        |
    ///  |         |   |        |
    ///  |         1---|-2------1
    ///  |    2      4 |   2
    ///  1------2------1
    ///
    ///
    /// ^ ih    > ic     / il
    // sum = 8*1 + 12*2 + 6*4 + 8 = 64
    fn smooth(&mut self) -> ReducedHsvHistogram {
        let mut ret = ReducedHsvHistogram {
            distribution : [[[0 ; 16] ; 16] ; 16],
            maxima : self.maxima.clone(),
        };
        for ih in 1..15 {
            for ic in 1..15 {
                for il in 1..15 {
                    ret.distribution[ih][ic][il] =
                        // top (ih += 1)
                        1 * 1 * 1 * self.distribution[ih+1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                        1 * 1 * 2 * self.distribution[ih+1][ic-1][il+0] +
                        1 * 1 * 1 * self.distribution[ih+1][ic-1][il+1] +                    // back (il += 1)
                        1 * 2 * 1 * self.distribution[ih+1][ic+0][il-1] + // mid ( ic += 0)  // front (il -=1)
                        1 * 2 * 2 * self.distribution[ih+1][ic+0][il+0] +
                        1 * 2 * 1 * self.distribution[ih+1][ic+0][il+1] +                    // back (il += 1)
                        1 * 1 * 1 * self.distribution[ih+1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                        1 * 1 * 2 * self.distribution[ih+1][ic+1][il+0] +
                        1 * 1 * 1 * self.distribution[ih+1][ic+1][il+1] +                    // back (il += 1)

                        // mid (ih += 0)
                        2 * 1 * 1 * self.distribution[ih+0][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                        2 * 1 * 2 * self.distribution[ih+0][ic-1][il+0] +
                        2 * 1 * 1 * self.distribution[ih+0][ic-1][il+1] +                    // back (il += 1)
                        2 * 2 * 1 * self.distribution[ih+0][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                        2 * 2 * 2 * self.distribution[ih+0][ic+0][il+0] +
                        2 * 2 * 1 * self.distribution[ih+0][ic+0][il+1] +                    // back (il += 1)
                        2 * 1 * 1 * self.distribution[ih+0][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                        2 * 1 * 2 * self.distribution[ih+0][ic+1][il+0] +
                        2 * 1 * 1 * self.distribution[ih+0][ic+1][il+1] +                    // back (il += 1)

                        // bot (ih -= 1)
                        1 * 1 * 1 * self.distribution[ih-1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                        1 * 1 * 2 * self.distribution[ih-1][ic-1][il+0] +
                        1 * 1 * 1 * self.distribution[ih-1][ic-1][il+1] +                    // back (il += 1)
                        1 * 2 * 1 * self.distribution[ih-1][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                        1 * 2 * 2 * self.distribution[ih-1][ic+0][il+0] +
                        1 * 2 * 1 * self.distribution[ih-1][ic+0][il+1] +                    // back (il += 1)
                        1 * 1 * 1 * self.distribution[ih-1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                        1 * 1 * 2 * self.distribution[ih-1][ic+1][il+0] +
                        1 * 1 * 1 * self.distribution[ih-1][ic+1][il+1] ;                    // back (il += 1)
                }
            }
        }
        ret
    }

    fn print(&self) {
       for ih in 0..16 {
            print!("\n\nh2:{}\n      l:    ", ih);
            for il in 0..16 {
                print!("{:4}", il);
            }
            for ic in 0..16 {
                print!("\n c2:{:4}  # ", ic);
                for il in 0..16 {
                    print!("{:4}", self.distribution[ih][ic][il]);
                }
                print!(" #");
            }
        }
        print!("\n");
    }
}

use std::path::{Path};
use std::fs::File;
use image;

#[test]
fn convert_and_back() {
    let img = image::open(&Path::new("assets/test/hsvtest.png")).unwrap();
    let mut hsv = HsvImage::from_image(&img);
    let hsvreduced = hsv.reduce_dynamic();
    //let hist = hsvreduced.histogram();

    let ref mut fout = File::create(&Path::new("out/hsvtest_convert_and_back.png")).unwrap();
    let _ = hsv.to_rgba().save(fout, image::PNG).unwrap();
    println!("\nPx 25,25 @ hsv     {:?}", hsv.get(25, 25));

    let ref mut fout = File::create(&Path::new("out/hsvtest_convert_and_back_reduced.png")).unwrap();
    let _ = hsvreduced.to_rgba().save(fout, image::PNG).unwrap();

    hsv.reduce_dynamic_self();
    println!("Px 25,25 @ hsvred2 {:?}", hsv.get(25, 25));
    //hsv.get(25, 25).to_rgba_print();
    let ref mut fout = File::create(&Path::new("out/hsvtest_convert_and_back_reduced2.png")).unwrap();
    let reduced_rgba = hsv.to_rgba();
    println!("Px 25,25 @ hsvred2 {:?}", reduced_rgba.get_pixel(25, 25));
    let _ = reduced_rgba.save(fout, image::PNG).unwrap();
}

#[test]
fn distance() {
    let red = Hsv::new(0, 255, 255, 255).reduce_dynamic();
    let orange = Hsv::new(20, 255, 255, 255).reduce_dynamic();
    //let dontknow = Hsv::new(20, 50, 255, 255).reduce_dynamic();
    println!("red {:?} to orange {:?}: {}", red, orange, red.distance(&orange));
    assert!(false);
}


#[test]
fn histogram() {
    let img = image::open(&Path::new("assets/emoticons2/1f30f.png")).unwrap();
    let hsv = HsvImage::from_image(&img);
    let hsvreduced = hsv.reduce_dynamic();
    let hist = hsvreduced.histogram();
    hist.print();
    assert!(false);

}