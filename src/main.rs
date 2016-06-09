

extern crate emotim;
use emotim::*;
use std::path::Path;

fn convert(file : &str, emoticons : &emoticons::Emoticons, frac : u32) {
    let mut ii = read_input_image(&format!("{}.jpg", file));
    let emoimg = Emoimage::new(&mut ii, frac, &emoticons, ComparisationMethod::Correlation);
    emoimg.save(&Path::new(&format!("out/{}.png", file)));
    println!("{}", emoimg);
}

fn main()  {
    println!("Hey");
    let emos = emoticons::read_emoticons();

    //convert("angels", &emos, 20);
    //convert("michelangelo", &emos, 25);
    //convert("monalisa", &emos, 25);
    //convert("perlenohrring", &emos, 25);
    convert("schrei", &emos, 15);
    //convert("sonnenblumen", &emos, 25);
    //convert("turmderblauenpferde", &emos, 25);

    let mut ii = read_input_image("schrei.jpg");
    let emoimg = Emoimage::new(&mut ii, 15, &emos, ComparisationMethod::Maxima);
    println!("{}", emoimg);
    emoimg.save(&Path::new("out/schrei_max.png"));

}
