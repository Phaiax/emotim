

extern crate emotim;
use emotim::*;
use std::path::Path;

fn main()  {
    println!("Hey");
    let emos = emoticons::read_emoticons();

    let mut ii = read_input_image("Munch_Schrei_6.jpg");
    let emoimg = Emoimage::new(&mut ii, 20, &emos, ComparisationMethod::Correlation);
    emoimg.save(&Path::new("out/munch_corr.png"));
    println!("{}", emoimg);

    let emoimg = Emoimage::new(&mut ii, 20, &emos, ComparisationMethod::Maxima);
    println!("{}", emoimg);
    emoimg.save(&Path::new("out/munch_max.png"));

}
