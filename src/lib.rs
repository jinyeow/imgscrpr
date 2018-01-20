extern crate clap;
extern crate reqwest;

use clap::ArgMatches;

use std::{env,fs,str};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
// use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use std::process;

pub mod provider;

#[derive(Debug)]
pub struct Options {
    // pub add_ordering: bool,
    // pub debug:        bool,
    // pub kpics:        bool,
    // pub nsfw:         bool,
    // pub output:       String,
    // pub sleep:        u32,
    // pub throttle:     u32,
    // pub title:        String,
    pub url:         String,
}

impl Options {
    pub fn new(matches: &ArgMatches) -> Result<Options, &'static str> {
        // TODO
        // write a test for this
        //  - with args
        //  - with no args
        //  - with conflicting args (e.g. nsfw + kpics, or title + > 1 url)

        let url = matches.value_of("URL").unwrap_or_else(|| {
            eprintln!("URL required");
            process::exit(1);
        });
        let url = String::from(url);

        // let Some(output)       = matches.value_of("output");
        // let Some(title)        = matches.value_of("title");
        // let Some(throttle)     = matches.value_of("throttle");
        // let Some(sleep)        = matches.value_of("sleep");
        // let Some(add_ordering) = matches.value_of("add-ordering");
        // let Some(debug)        = matches.value_of("debug");
        // let Some(nsfw)         = matches.value_of("nsfw");
        // let Some(kpics)        = matches.value_of("kpics");

        Ok(Options { url })
    }
}

pub fn run(opts: Options) -> Result<(), Box<Error>> {
    // TODO

    println!("{:?}", opts);

    Ok(())
}

pub fn mk_cd_dir(location: &str) {
    fs::create_dir(&location);

    env::set_current_dir(&location);
    println!("\t-- chdir {}", location);
}

pub fn mkdir_default() {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/scraped/", home.display());
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location);
}

pub fn mkdir_custom(dir: &str) {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/{}/", home.display(), dir);
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location);
}

pub fn mkdir_custom_with_title(dir: &str, title: &str) {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/{}/{}", home.display(), dir, title);
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location);
}

pub fn uniq_valid_filename(f: &str) -> String {
    let filename = f.replace(r"/", "_");

    let mut duplicate_num = 0;
    let mut path = PathBuf::from(&filename);
    let mut namae = String::from(path.file_stem().unwrap().to_str().unwrap());
    let mut n = String::from(filename);

    let p = path.clone();
    let ext = p.extension().unwrap().to_str().unwrap();

    while path.exists() {
        duplicate_num = duplicate_num + 1;
        n = format!("{}_{}.{}", namae, duplicate_num, ext);
        path = PathBuf::from(&n);
    }

    String::from(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::path::Path;
    use clap::{App,Arg};

    #[test]
    fn it_returns_a_unique_filename() {
        let filename = "test.txt";
        File::create(&filename);
        let uniq_filename = uniq_valid_filename(&filename);
        fs::remove_file(&filename);

        assert_ne!(filename, uniq_filename);
        assert_eq!("test_1.txt", uniq_filename);
    }

    #[test]
    fn it_returns_a_valid_filename() {
        let filename = "te/st.txt";
        let valid_filename = uniq_valid_filename(&filename);

        assert_ne!(valid_filename, filename);
        assert_eq!(valid_filename, "te_st.txt");
    }

    #[test]
    fn it_returns_a_unique_valid_filename() {
        let filename = "te/st/file.txt";

        let valid_filename = uniq_valid_filename(&filename);
        File::create(&valid_filename);

        assert_ne!(valid_filename, filename);
        assert_eq!(valid_filename, "te_st_file.txt");
        assert!(Path::new(&valid_filename).exists());

        let unique_filename = uniq_valid_filename(&filename);
        File::create(&unique_filename);

        assert_ne!(unique_filename, valid_filename);
        assert_eq!(unique_filename, "te_st_file_1.txt");

        let unique_filename_2 = uniq_valid_filename(&filename);

        assert_ne!(unique_filename_2, unique_filename);

        fs::remove_file(&valid_filename);
        fs::remove_file(&unique_filename);

        assert_eq!(unique_filename_2, "te_st_file_2.txt");
    }

    #[test]
    fn it_returns_a_valid_options_struct() {
        let args = vec!["test", "www.example.com"];

        let matches = App::new("test")
                              .version("0.0.1")
                              .author("Test Author")
                              .about("Test Description")
                              .arg(Arg::with_name("first")
                                   .short("f")
                                   .long("first")
                                   .value_name("FIRST")
                                   .takes_value(true))
                              .arg(Arg::with_name("URL")
                                   .required(true)
                                   .index(1))
                              .get_matches_from(args);

        let opts = Options::new(&matches).unwrap();

        assert_eq!(opts.url, "www.example.com");
    }
}
