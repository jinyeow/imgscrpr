extern crate clap;
extern crate regex;
extern crate reqwest;
extern crate scraper;
extern crate url;

#[macro_use]
extern crate serde_json;

// Crate Modules
use clap::ArgMatches;
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use url::Url;

// Stdlib Modules
use std::{env, fs, str};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use std::process;

// Custom
pub mod provider;

use provider::imgur;
use provider::gfycat;

#[derive(Debug)]
pub struct UnsupportedUrlError;

impl Error for UnsupportedUrlError {
    fn description(&self) -> &str {
        "[!!] URL not supported"
    }
}

impl fmt::Display for UnsupportedUrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnsupportedUrlError")
    }
}

#[derive(Debug)]
pub struct Options {
    // pub add_ordering: bool,
    pub debug:        bool,
    pub kpics:        bool,
    pub nsfw:         bool,
    pub output:       String,
    // pub sleep:        u32,
    // pub throttle:     u32,
    pub title:        String,
    pub urls:         Vec<String>,
}

impl Options {
    pub fn new(matches: &ArgMatches) -> Result<Options, &'static str> {
        if matches.occurrences_of("URL") > 1 && matches.is_present("title") {
            return Err("Conflicting options: '--title' and multiple URLs");
        }

        let urls: Vec<_> = matches.values_of("URL").unwrap().collect();
        let urls = urls.iter().map(|&x| {
            String::from(x)
        }).collect();

        let title = match matches.value_of("title") {
            Some(t) => String::from(t),
            None => String::from("")
        };

        let output = match matches.value_of("output") {
            Some(o) => String::from(o),
            None => String::from("")
        };

        let debug = if matches.occurrences_of("debug") > 0 {
            true
        } else {
            false
        };

        let nsfw = if matches.occurrences_of("nsfw") > 0 {
            true
        } else {
            false
        };

        let kpics = if matches.occurrences_of("kpics") > 0 {
            true
        } else {
            false
        };

        // let Some(throttle)     = matches.value_of("throttle");
        // let Some(sleep)        = matches.value_of("sleep");
        // let Some(add_ordering) = matches.value_of("add-ordering");

        Ok(Options { debug, kpics, nsfw, output, title, urls, })
    }
}

pub fn run(opts: Options) -> Result<(), Box<Error>> {
    let ext_re     = Regex::new(r"\.(\w+)$").unwrap();
    let imgur_re   = Regex::new(r"imgur").unwrap();
    let gfycat_re  = RegexBuilder::new(r"gfycat")
        .case_insensitive(true)
        .build()
        .unwrap();
    let general_re = Regex::new(r"/([a-zA-Z0-9\-_]+)\.(\w+)$").unwrap();

    for url in opts.urls {
        if !is_valid_url(&url) {
            continue;
        }

        let mut count  = 0;
        let mut failed = 0;
        let host       = host(&url);

        let data:        Value;
        let mut sub_dir: Option<String>;
        let dir:         Option<&str>;

        print!("[+] Scraping {}: ", url);

        if imgur_re.is_match(&host) {
            data = match imgur::scrape_data(&url) {
                Ok(i) => i,
                Err(e) => return Err(e)
            };
        } else if gfycat_re.is_match(&url) {
            data = match gfycat::scrape_data(&url) {
                Ok(g) => g,
                Err(e) => return Err(e)
            };
        } else if general_re.is_match(&url) {
            let captures = general_re.captures(&url).unwrap();
            data         = json!({
                "link": url,
                "id":   captures[1],
                "ext":  captures[2]
            });
        } else {
            return Err(Box::new(UnsupportedUrlError))
        }

        if opts.debug {
            println!("{:?}", data);
            process::exit(0);
        }

        if data["title"] != json!(null) {
            sub_dir = Some(String::from(data["title"].as_str().unwrap()))
        } else {
            sub_dir = None
        };

        println!("{}...complete!", if sub_dir.is_some() {
            sub_dir.clone().unwrap()
        } else {
            String::from("")
        });

        // array of images
        let images = if data["images"] != json!(null) {
            data["images"].as_array().unwrap().to_vec()
        } else {
            vec![data]
        };

        println!("\t-- found {} image{}", images.len(), if images.len() > 1 {
                                                            "s"
                                                        } else {
                                                            ""
                                                        });

        // Choose output directory depending on flags
        if opts.output != "" {
            sub_dir = Some(opts.output.clone())
        } else if sub_dir.is_none() {
            sub_dir = None
        };

        if opts.nsfw {
            dir = Some("nsfw");
        } else if opts.kpics {
            dir = Some("kpics");
        } else {
            dir = None;
        }

        if dir != None && sub_dir != None {
            mkdir_custom_with_sub_dir(&dir.unwrap(), &sub_dir.unwrap())?;
        } else if dir != None {
            mkdir_custom(dir.unwrap())?;
        } else if sub_dir != None {
            mkdir_custom_with_sub_dir("scraped", &sub_dir.unwrap())?;
        } else {
            mkdir_default()?;
        }

        // SCRAPE
        println!("  [+] Scraping images now...");
        let mut i = 0;
        for img in images.iter() {
            i += 1;

            let filetype;
            let mut filename;

            if img["ext"] != json!(null) {
                filetype = String::from(img["ext"].as_str().unwrap());
            } else {
                filetype = String::from(
                    &ext_re.captures(img["link"].as_str().unwrap())
                    .unwrap()[1]
                );
            }

            if opts.title.len() > 0 {
                filename = opts.title.clone();
            } else if img["title"] != json!(null) {
                filename = String::from(img["title"].as_str().unwrap());
            } else {
                filename = String::from(img["id"].as_str().unwrap());
            }
            filename = str::replace(
                &format!("{}.{}", filename, filetype), " ", "_"
            );

            let filename = uniq_valid_filename(&filename);

            print!("\t{:4}. {}...", i, filename);

            // begin scrape
            match scrape(img, &filename) {
                Ok(_) => {
                    println!("done!");
                    count += 1;
                },
                Err(_) =>  {
                    println!("failed");
                    fs::remove_file(filename)?;
                    failed += 1;
                }
            };

            // TODO
            // if opts.throttle > 0 && img != images.last {
            //     sleep opts.throttle
            // }
        }

        let count_str = format!("{} image{}", count, if count > 1 { "s" } else { "" });
        println!(
            "  [*] Finished scraping {}: {} successfully scraped.", url, count_str
        );
        if failed > 0 { println!("\tScrape failed: {} images.", failed); }
        // TODO
        // if url != last_url { println!("\n"); }
    }

    Ok(())
}

fn scrape(img: &Value, filename: &str) -> Result<(), Box<Error>> {
    let url = img["link"].as_str().unwrap();
    let mut data = reqwest::get(url)?;
    let mut f = File::create(String::from(filename))?;

    // f.write_all(data)?;
    data.copy_to(&mut f)?;

    Ok(())
}

fn mk_cd_dir(location: &str) -> Result<(), Box<Error>> {
    fs::create_dir_all(&location)?;

    env::set_current_dir(&location)?;
    println!("\t-- chdir {}", location);

    Ok(())
}

// TODO: refactor mkdir_*() to take Option args.
//  That way we only need 1 function to do the work of 3.
fn mkdir_default() -> Result<(), Box<Error>> {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/scraped/", home.display());
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location)?;

    Ok(())
}

fn mkdir_custom(dir: &str) -> Result<(), Box<Error>> {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/{}/", home.display(), dir);
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location)?;

    Ok(())
}

fn mkdir_custom_with_sub_dir(dir: &str, sub_dir: &str) -> Result<(), Box<Error>> {
    let home = env::home_dir().unwrap();
    let location = format!("{}/Pictures/{}/{}", home.display(), dir, sub_dir);
    println!("\t-- saving images to {}", location);

    mk_cd_dir(&location)?;

    Ok(())
}

fn uniq_valid_filename(f: &str) -> String {
    let filename = f.replace(r"/", "_");

    let mut duplicate_num = 0;
    let mut path = PathBuf::from(&filename);
    let namae = String::from(path.file_stem().unwrap().to_str().unwrap());
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

fn is_valid_url(url: &str) -> bool {
    let uri;
    match Url::parse(url) {
        Ok(u) => uri = u,
        Err(_) => return false
    };

    let result = ["http", "https"].iter().any(|x| {
        x == &uri.scheme()
    });

    result
}

fn host(url: &str) -> String {
    let h = match Url::parse(url).unwrap().host_str() {
        Some(s) => String::from(s),
        None => String::from("")
    };
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::{App, Arg};

    use std::fs;
    use std::fs::File;
    use std::path::Path;

    #[test]
    fn it_returns_a_unique_filename() {
        let filename = "test.txt";
        File::create(&filename).expect(
            "Couldn't create file in test: 'it_returns_a_unique_filename'"
            );
        let uniq_filename = uniq_valid_filename(&filename);
        fs::remove_file(&filename).expect(
            "Couldn't remove file in test: 'it_returns_a_unique_filename'"
            );

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
        File::create(&valid_filename).expect(
            "Couldn't create file in test: 'it_returns_a_unique_valid_filename'"
            );

        assert_ne!(valid_filename, filename);
        assert_eq!(valid_filename, "te_st_file.txt");
        assert!(Path::new(&valid_filename).exists());

        let unique_filename = uniq_valid_filename(&filename);
        File::create(&unique_filename).expect(
            "Couldn't create file in test: 'it_returns_a_unique_valid_filename'"
            );

        assert_ne!(unique_filename, valid_filename);
        assert_eq!(unique_filename, "te_st_file_1.txt");

        let unique_filename_2 = uniq_valid_filename(&filename);

        assert_ne!(unique_filename_2, unique_filename);

        fs::remove_file(&valid_filename).expect(
            "Couldn't remove file in test: 'it_returns_a_unique_valid_filename'"
            );
        fs::remove_file(&unique_filename).expect(
            "Couldn't remove file in test: 'it_returns_a_unique_valid_filename'"
            );

        assert_eq!(unique_filename_2, "te_st_file_2.txt");
    }

    #[test]
    fn it_returns_a_options_from_valid_args() {
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

        assert_eq!(opts.urls, ["www.example.com"]);
    }

    #[test]
    fn it_returns_err_with_conflicting_args_title_and_multiple_urls() {
        let args = vec![
            "test",
            "www.example.com",
            "www.foobar.com",
            "--title",
            "title"
        ];

        let matches = App::new("test")
                              .version("0.0.1")
                              .author("Test Author")
                              .about("Test Description")
                              .arg(Arg::with_name("title")
                                   .short("t")
                                   .long("title"))
                              .arg(Arg::with_name("URL")
                                   .required(true)
                                   .multiple(true)
                                   .index(1))
                              .get_matches_from(args);

        let result = Options::new(&matches);

        assert!(result.is_err());
    }

    #[test]
    fn it_returns_true_on_valid_url() {
        let url = "https://github.com";
        let result = is_valid_url(url);
        assert!(result);
    }

    #[test]
    fn it_returns_false_on_invalid_url() {
        let url = "not.valid/url";
        let result = is_valid_url(url);
        assert!(!result);
    }

    #[test]
    fn it_returns_false_on_empty_url() {
        let url = "";
        let result = is_valid_url(url);
        assert!(!result);
    }

    // TODO: add test for other sites/providers as well
    #[test]
    fn it_creates_an_image_from_a_valid_link() {
        let valid_link = "https://i.imgur.com/0rut99n.jpg";
        let filename = "0rut99n.jpg";

        let img = json!({"link": valid_link});

        let result = scrape(&img, &filename);

        assert!(Path::new(filename).exists());

        // remove the created image from scrape()
        fs::remove_file(filename).expect(
            "Couldn't remove file in test: 'it_creates_an_image_from_a_valid_link'"
            );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn it_returns_err_and_does_not_create_an_image_on_invalid_link() {
        let invalid_link = "https://i.imgur.not/aValidL.ink";
        let filename = "aValidL.ink";
        let img = json!({"link": invalid_link});
        let result = scrape(&img, &filename);

        assert!(!Path::new(filename).exists());
        assert!(result.is_err());
    }
}
