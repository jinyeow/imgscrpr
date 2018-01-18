extern crate reqwest;

pub mod imgscrpr {
    // TODO
}

pub mod imgur {
    use reqwest;
    use reqwest::{Response,Error};
    use std::{env,fs,str};
    use std::fs::File;
    // use std::ffi::OsStr;
    use std::path::Path;
    use std::path::PathBuf;
    use std::io::prelude::*;

    const API_VERSION: i32 = 3;
    const CLIENT_ID: &str = "b5f97137be15df2";

    pub fn get_data(id: &str, collection: bool) -> Result<Response, Error> {
        let t = if collection {
            "album"
        } else {
            "image"
        };
        let api_url = format!("https//api.imgur.com/{}/{}/{}", CLIENT_ID, t, id);
        reqwest::get(&api_url)
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

    // pub fn scrape(img: &Image, filename: &str) {
    //     let mut file = File::create(&filename).unwrap();
    //     file.write(content);
    // }

    pub fn uniq_valid_filename(f: &str) -> String {
        let filename = f.replace(r"/", "_");

        let mut duplicate_num = 0;
        let mut path = PathBuf::from(&filename);
        let mut namae = String::from(path.file_stem().unwrap().to_str().unwrap());

        let p = path.clone();
        let ext = p.extension().unwrap().to_str().unwrap();

        while path.exists() {
            duplicate_num = duplicate_num + 1;
            namae = format!("{}_{}", namae, duplicate_num);
            path = PathBuf::from(&namae);
        }

        String::from(format!("{}.{}", namae, ext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_returns_a_unique_filename() {
        let filename = "test.txt";
        File::create(&filename);
        let uniq_filename = imgur::uniq_valid_filename(&filename);
        fs::remove_file(&filename);

        assert_ne!(filename, uniq_filename);
    }

    #[test]
    fn it_returns_a_valid_filename() {
        let filename = "te/st.txt";
        let valid_filename = imgur::uniq_valid_filename(&filename);

        assert_ne!(valid_filename, filename);
        assert_eq!(valid_filename, "te_st.txt");
    }
}
