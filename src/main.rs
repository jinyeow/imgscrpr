extern crate imgscrpr;

extern crate clap;

use imgscrpr::Options;

use clap::{Arg, App};

use std::process;

fn main() {
    let matches = App::new("imgscrpr")
                          .version("0.0.1")
                          .author("Justin P. <jin-yeow@outlook.com>")
                          .about("Scrapes images given url(s).")
                          .arg(Arg::with_name("output")
                               .short("o")
                               .long("output")
                               .value_name("DIRECTORY")
                               .help("Specify directory to scrape image(s) to")
                               .takes_value(true))
                          .arg(Arg::with_name("URL")
                               .help("Specifies image url.")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("title")
                               .short("t")
                               .long("title")
                               .value_name("TITLE")
                               .help("Specify filename for a single image")
                               .takes_value(true))
                          .arg(Arg::with_name("throttle")
                               .long("throttle")
                               .value_name("TIME")
                               .help("Wait TIME seconds between downloads")
                               .takes_value(true))
                          .arg(Arg::with_name("sleep")
                               .long("sleep")
                               .value_name("TIME")
                               .help("Wait TIME seconds between sets of images")
                               .takes_value(true))
                          .arg(Arg::with_name("add-ordering")
                               .long("add-ordering")
                               .help("Prepend ordered number to filenames"))
                          .arg(Arg::with_name("debug")
                               .short("d")
                               .long("debug")
                               .help("Print out image data for debugging purposes"))
                          .arg(Arg::with_name("nsfw")
                               .short("n")
                               .long("nsfw")
                               .help("Use the 'nsfw' directory instead of the default"))
                          .arg(Arg::with_name("kpics")
                               .short("k")
                               .long("kpics")
                               .help("Use the 'kpics' directory instead of the default"))
                          .get_matches();

    let opts = Options::new(&matches).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = imgscrpr::run(opts) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
