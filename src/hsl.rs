//! This module provides functions to convert `DynamicImage`s to its Hsl representation.
//! It then allows to reduce the color spectrum from 255^3 to 16^3 = 4096 different colors .
//! From these images with reduced depth, it can generate a 3 dimensional histogram.
//!
//! Hsl is a pixel representation consisting of three values:
//!
//!  * Hue (color angle)
//!  * Chroma (Saturation)
//!  * Lightness
//!
//! (with the current implementation it should better be called HSL)
//!
//! See [wikipedias HSL_and_HSV](https://en.wikipedia.org/wiki/HSL_and_HSV) for more information.


use image::{Pixel, Rgba, GenericImage, DynamicImage, RgbaImage};
use std::f32;


/// HSL pixel
///
/// Skip hexagon to circle transformation and use simplified conversion:
///
///  * alpha = 0.5 * (2 R - G - B)
///  * beta = sqrt(3) / 2 * (G - B)
///  * h2 = atan( beta, alpha )
///  * c2 = sqrt( alpha^2 * beta^2 )
///  * l = 0.5 * (m + M)
///  * Copy alpha from rgba
///
/// If `Hsl` represents a full depth pixel, all values range from 0 to 255.
///
/// If it represents a pixel with reduced depts, `h2`, `c2` and `l` range from 0 to 16 and `a` can be 0 or one.
///
/// If the lightness is at 0 or 255, the hue has no meaning.
/// If the chroma is at 0, the hue has no meaning (grey values).
#[derive(Debug,Clone,PartialEq)]
pub struct Hsl {
    /// Hue (color angle: 0: red, 255: red, 128: turquoise, 64: green)
    pub h2 : u8,
    /// Chroma (Saturation)
    pub c2 : u8,
    /// Lightness (0: black, 255: white)
    pub l : u8,
    /// Alpha (255: visible)
    pub a : u8,
}

impl Hsl {
    /// Creates a new pixel from the given values.
    pub fn new(h2 : u8, c2 : u8, l : u8, a : u8) -> Hsl {
        Hsl {h2 : h2, c2 : c2, l : l , a : a }
    }

    /// Calculates a color distance. The uses formula has apparent problems.
    ///
    /// TODO:
    ///
    /// * hue needs wrapping
    /// * hues influence must depend on chroma and lightness
    pub fn distance(&self, other : &Hsl) -> u32 {
        ((self.h2 as i32 - other.h2 as i32).abs() +
        (self.c2 as i32 - other.c2 as i32).abs() +
        (self.l as i32 - other.l as i32).abs() +
        (self.a as i32 - other.a as i32).abs() ) as u32
    }

    /// Converts this pixel back into RGBA color space. The conversion is not loseless.
    /// This works on full depth `Hsl` pixels.
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
        // Prevent wrapping
        if m < 0. {
            if r1 < -m { r1 = -m };
            if g1 < -m { g1 = -m };
            if b1 < -m { b1 = -m };
        }
        // Prevent wrapping
        if m > 0. {
            if r1 + m > 255. { r1 = 255.-m };
            if g1 + m > 255. { g1 = 255.-m };
            if b1 + m > 255. { b1 = 255.-m };
        }
        Rgba::from_channels((r1+m) as u8, (g1+m) as u8, (b1+m) as u8, self.a)
    }

    /// Extend the dynamic (aka color depth) of this pixel by multiplying with 16 (or 255 for alpha).
    pub fn extend_dynamic(&self) -> Hsl {
        Hsl { h2 : self.h2 * 16,
              c2 : self.c2 * 16,
              l : self.l * 16,
              a : self.a * 255}
    }

    /// Reduce the dynamic (aka color depth) of this pixel by dividing by 16 (or 255 for alpha).
    pub fn reduce_dynamic(&self) -> Hsl {
        Hsl {
            h2 : self.h2 / 16,
            c2 : self.c2 / 16,
            l : self.l / 16,
            a : if self.a > 204 { 1 } else { 0 },
        }
    }
}

impl From<Rgba<u8>> for Hsl {
    /// Converts an `Rgba` pixel into an `Hsl` pixel
    fn from(pixel : Rgba<u8>) -> Hsl {
        let (r, g, b, a) = pixel.channels4();
        let r = r as f32;
        let g = g as f32;
        let b = b as f32;
        let alpha : f32 = 0.5f32 * ( 2f32 * r - g - b);
        let beta : f32 = 3f32.sqrt() / 2f32 * (g - b);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        Hsl {
            h2 : (beta.atan2(alpha) * 128.0 / f32::consts::PI) as u8,
            c2 : ( (alpha.powi(2) + beta.powi(2)).sqrt()  ) as u8,
            l : ((max + min) / 2.0) as u8,
            a : a,
        }
    }
}

/// An image consisting of HSL pixels
pub struct HslImage {
    /// The pixels. This vec has len width*height.
    pub pixels : Vec<Hsl>,
    /// Height of this image
    pub height : u32,
    /// Width of this image
    pub width : u32,
}


impl HslImage {

    /// Convert RGBA image into HSL color space
    pub fn from_image<T>(rgba_img : &T) -> HslImage
        where T : GenericImage<Pixel = Rgba<u8>> {

        let size = (rgba_img.width() * rgba_img.height()) as usize;
        let mut hslpixels = Vec::with_capacity( size );
        for (_,_,pixel) in rgba_img.pixels() {
            hslpixels.push( Hsl::from(pixel) );
        }

        HslImage {
            pixels : hslpixels,
            height : rgba_img.height(),
            width : rgba_img.width(),
        }
    }

    /// Convert into RGBA color space
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

    /// Convert full depth HSL color space image into reduced depth HSL color space image
    /// with a total of 16^3 = 4096 different colors.
    ///
    /// * h2: 16 steps
    /// * c2: 16 steps
    /// * l : 16 steps
    /// * alpha : 2 values: On (>80%), Off (<80%)
    ///
    pub fn reduce_dynamic(&self) -> HslImageWithReducedDepth {
        let mut hsl = Vec::with_capacity(self.pixels.len());

        for h in &self.pixels {
            hsl.push( h.reduce_dynamic() );
        }

        HslImageWithReducedDepth(HslImage {
            pixels : hsl,
            height : self.height,
            width : self.width,
        })
    }

    /// Get Pixel value at `(x, y)`
    pub fn get(&self, x : u32, y : u32) -> Hsl {
        self.pixels[( y * self.width + x ) as usize ].clone()
    }

}


/// A HSL color space image with reduced depth color space.
///
/// Can be obtained by calling `reduce_dynamic()` on a `HslImage`.
pub struct HslImageWithReducedDepth (HslImage);


impl HslImageWithReducedDepth {

    /// Convert reduced depth HSL color space image into a full depth HSL color space image.
    pub fn extend_dynamic(&self) -> HslImage {
        let mut hsl = Vec::with_capacity(self.0.pixels.len());

        for h in &self.0.pixels {
            hsl.push( h.extend_dynamic() );
        }

        HslImage {
            pixels : hsl,
            height : self.0.height,
            width : self.0.width,
        }
    }

    /// Calculate a histogram, smooth it and find local maxima
    pub fn histogram(&self) -> HslHistogram {
        HslHistogram::from_reduced_depth_hsl_image(&self)
    }

}

/// Histogram over all colors of a reduced color depth HSL image.
pub struct HslHistogram {
    /// Distribution of the colors in three dimensional color space (4096 colors).
    ///
    /// Index via: distribution[h2][c2][l]
    pub distribution : [[[u32 ; 16] ; 16] ; 16],
    /// Gaussian 3d smoothed (28n kernel) color distribution
    ///
    /// Index via: smoothed[h2][c2][l]
    pub smoothed : [[[u32 ; 16] ; 16] ; 16],
    /// List of significant local maxima in the smoothed histogram.
    /// The first tuple gives the color (aka position), the second tuple gives an estimate of
    /// the number of pixels that have this or a similar color.
    pub maxima : Vec<(Hsl, f32)>,
}

impl HslHistogram {

    /// Calculate a histogram, smooth it and find local maxima
    pub fn from_reduced_depth_hsl_image(img : &HslImageWithReducedDepth) -> HslHistogram {
        let mut ret = HslHistogram {
            distribution : [[[0 ; 16] ; 16] ; 16],
            smoothed     : [[[0 ; 16] ; 16] ; 16],
            maxima       : Vec::with_capacity(5),
        };
        for h in &img.0.pixels {
            if h.a == 0 { // check alpha (a can only be 0 or 1 in reduced color space)
                continue;
            }
            ret.distribution[h.h2 as usize][h.c2 as usize][h.l as usize] += 1;
        }
        ret.smooth();
        ret.find_maxima();
        ret
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
    ///
    /// sum = 8*1 + 12*2 + 6*4 + 8 = 64
    fn smooth(&mut self) {
        for ih in 1..15 {
            for ic in 1..15 {
                for il in 1..15 {
                    self.smoothed[ih][ic][il] =
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
    }

    /// Finds maxima within the smoothed histogram.
    ///
    /// Strategy:
    /// Look at all 27 neighbours of a non-border color. Set as maxima if no
    /// neighbours are greater. (For neighbours to the right-bot-back direction,
    /// use >= instead of >, so that two equal values will only generate one maximum).
    ///
    /// After finding maximas, calculate the size of the corresponding maxima
    /// by adding up all direct neighbours values. Take into account that the smoothed
    /// values are not normalized: divide by 64
    ///
    /// Sore the maximas. Only keep 5. Discard little maximas.
    fn find_maxima(&mut self) {

        for ih in 1..15 {
            for ic in 1..15 {
                for il in 1..15 {
                    let center = self.smoothed[ih][ic][il];
                    if center == 0 { continue; }
                    let found_greater =
                        // top (ih += 1)
                             if self.smoothed[ih+1][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if self.smoothed[ih+1][ic-1][il+0] > center { true }
                        else if self.smoothed[ih+1][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if self.smoothed[ih+1][ic+0][il-1] > center { true } // mid ( ic += 0)  // front (il -=1)
                        else if self.smoothed[ih+1][ic+0][il+0] > center { true }
                        else if self.smoothed[ih+1][ic+0][il+1] > center { true }                    // back (il += 1)
                        else if self.smoothed[ih+1][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if self.smoothed[ih+1][ic+1][il+0] > center { true }
                        else if self.smoothed[ih+1][ic+1][il+1] > center { true }                    // back (il += 1)

                        // mid (ih += 0)
                        else if self.smoothed[ih+0][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if self.smoothed[ih+0][ic-1][il+0] > center { true }
                        else if self.smoothed[ih+0][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if self.smoothed[ih+0][ic+0][il-1] > center { true } // mid ( ic += 0) // front (il -=1)
                        //else if self.smoothed[ih+0][ic+0][il+0] > center { true }
                        else if self.smoothed[ih+0][ic+0][il+1] >= center { true }                    // back (il += 1)
                        else if self.smoothed[ih+0][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if self.smoothed[ih+0][ic+1][il+0] >= center { true }
                        else if self.smoothed[ih+0][ic+1][il+1] >= center { true }                    // back (il += 1)

                        // bot (ih -= 1)
                        else if self.smoothed[ih-1][ic-1][il-1] > center { true } // left ( ic -= 1) // front (il -=1)
                        else if self.smoothed[ih-1][ic-1][il+0] > center { true }
                        else if self.smoothed[ih-1][ic-1][il+1] > center { true }                    // back (il += 1)
                        else if self.smoothed[ih-1][ic+0][il-1] > center { true } // mid ( ic += 0) // front (il -=1)
                        else if self.smoothed[ih-1][ic+0][il+0] >= center { true }
                        else if self.smoothed[ih-1][ic+0][il+1] >= center { true }                    // back (il += 1)
                        else if self.smoothed[ih-1][ic+1][il-1] > center { true } // right ( ic += 1) // front (il -=1)
                        else if self.smoothed[ih-1][ic+1][il+0] >= center { true }
                        else if self.smoothed[ih-1][ic+1][il+1] >= center { true }                    // back (il += 1)
                        else { false } ;

                    if ! found_greater {
                        let sum =
                            // top (ih += 1)
                            self.smoothed[ih+1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            self.smoothed[ih+1][ic-1][il+0] +
                            self.smoothed[ih+1][ic-1][il+1] +                    // back (il += 1)
                            self.smoothed[ih+1][ic+0][il-1] + // mid ( ic += 0)  // front (il -=1)
                            self.smoothed[ih+1][ic+0][il+0] +
                            self.smoothed[ih+1][ic+0][il+1] +                    // back (il += 1)
                            self.smoothed[ih+1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            self.smoothed[ih+1][ic+1][il+0] +
                            self.smoothed[ih+1][ic+1][il+1] +                    // back (il += 1)

                            // mid (ih += 0)
                            self.smoothed[ih+0][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            self.smoothed[ih+0][ic-1][il+0] +
                            self.smoothed[ih+0][ic-1][il+1] +                    // back (il += 1)
                            self.smoothed[ih+0][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                            self.smoothed[ih+0][ic+0][il+0] +
                            self.smoothed[ih+0][ic+0][il+1] +                    // back (il += 1)
                            self.smoothed[ih+0][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            self.smoothed[ih+0][ic+1][il+0] +
                            self.smoothed[ih+0][ic+1][il+1] +                    // back (il += 1)

                            // bot (ih -= 1)
                            self.smoothed[ih-1][ic-1][il-1] + // left ( ic -= 1) // front (il -=1)
                            self.smoothed[ih-1][ic-1][il+0] +
                            self.smoothed[ih-1][ic-1][il+1] +                    // back (il += 1)
                            self.smoothed[ih-1][ic+0][il-1] + // mid ( ic += 0) // front (il -=1)
                            self.smoothed[ih-1][ic+0][il+0] +
                            self.smoothed[ih-1][ic+0][il+1] +                    // back (il += 1)
                            self.smoothed[ih-1][ic+1][il-1] + // right ( ic += 1) // front (il -=1)
                            self.smoothed[ih-1][ic+1][il+0] +
                            self.smoothed[ih-1][ic+1][il+1] ;                    // back (il += 1)

                        self.maxima.push((Hsl{
                                                h2 : ih as u8,
                                                c2 : ic as u8,
                                                l  : il as u8,
                                                a  : 1
                                          },
                                          sum as f32 / 64. )); // 64 is the sum of the smooth factors for all neighbours
                    }

                }
            }
        }
        self.sort_maxima();
    }

    /// Sort the maxima. Smallest first. Only keep 5. Discard little maximas.
    fn sort_maxima(&mut self) {
        self.maxima.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        while self.maxima.len() > 5 || (self.maxima.len() >= 1 && self.maxima.first().unwrap().1 < 1. ) {
            self.maxima.remove(0);
        }
    }

    /// Calculate similarity between two histograms by comparing maxima.
    pub fn similarity_by_maxima(&self, other : &HslHistogram) -> f32 {
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

    /// Calculate similarity between two histogramms by correlating them.
    pub fn similarity_by_correlation(&self, other : &HslHistogram) -> f32 {
        let mut correlation = 0.0;
        for ih in 0..16 {
            for ic in 0..16 {
                for il in 0..16 {
                    correlation += self.smoothed[ih][ic][il] as f32 *
                                   other.smoothed[ih][ic][il] as f32;
                }
            }
        }
        correlation
    }

    /// Prints this histogramm to stdout.
    pub fn print(&self) {
       for ih in 0..16 {
            print!("\n\nh2:{}\n      l:    ", ih);
            for il in 0..16 {
                print!("{:4}", il);
            }
            for ic in 0..16 {
                print!("\n c2:{:4}  # ", ic);
                for il in 0..16 {
                    print!("{:4}", self.smoothed[ih][ic][il]);
                }
                print!(" #");
            }
        }
        print!("\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path};
    use std::fs::File;
    use image;

    #[test]
    fn convert_and_back() {
        let img = image::open(&Path::new("assets/test/hsltest.png")).unwrap();
        let mut hsl = HslImage::from_image(&img);
        let hslreduced = hsl.reduce_dynamic();
        //let hist = hslreduced.histogram();

        let ref mut fout = File::create(&Path::new("out/hsltest_convert_and_back.png")).unwrap();
        let _ = hsl.to_rgba().save(fout, image::PNG).unwrap();
        println!("\nPx 25,25 @ hsl     {:?}", hsl.get(25, 25));

        let ref mut fout = File::create(&Path::new("out/hsltest_convert_and_back_reduced.png")).unwrap();
        let _ = hslreduced.to_rgba().save(fout, image::PNG).unwrap();

        hsl.reduce_dynamic_self();
        println!("Px 25,25 @ hslred2 {:?}", hsl.get(25, 25));
        //hsl.get(25, 25).to_rgba_print();
        let ref mut fout = File::create(&Path::new("out/hsltest_convert_and_back_reduced2.png")).unwrap();
        let reduced_rgba = hsl.to_rgba();
        println!("Px 25,25 @ hslred2 {:?}", reduced_rgba.get_pixel(25, 25));
        let _ = reduced_rgba.save(fout, image::PNG).unwrap();
    }

    #[test]
    fn distance() {
        let red = Hsl::new(0, 255, 255, 255).reduce_dynamic();
        let orange = Hsl::new(20, 255, 255, 255).reduce_dynamic();
        //let dontknow = Hsl::new(20, 50, 255, 255).reduce_dynamic();
        println!("red {:?} to orange {:?}: {}", red, orange, red.distance(&orange));
        assert!(false);
    }


    #[test]
    fn histogram() {
        let img = image::open(&Path::new("assets/emoticons2/1f30f.png")).unwrap();
        let hsl = HslImage::from_image(&img);
        let hslreduced = hsl.reduce_dynamic();
        let hist = hslreduced.histogram();
        hist.print();
        assert!(false);

    }
}