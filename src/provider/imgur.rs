use regex::Regex;
use reqwest;
use reqwest::Response;
use reqwest::header::Authorization;
use serde_json;
use serde_json::Value;

use std::error::Error;

const API_VERSION: i32 = 3;
const CLIENT_ID: &str = "b5f97137be15df2";

fn get_data(id: &str, collection: bool) -> Result<Response, reqwest::Error> {
    let t = if collection {
        "album"
    } else {
        "image"
    };
    let api_url = format!("https://api.imgur.com/{}/{}/{}", API_VERSION, t, id);

    let client = reqwest::Client::new();
    client.get(&api_url)
        .header(Authorization(format!("Client-ID {}", CLIENT_ID)))
        .send()
}

fn is_collection(url: &str) -> bool {
    let re = Regex::new(r"/a/|/gallery/").unwrap();
    re.is_match(url)
}

pub fn scrape_data(url: &str) -> Result<Value, Box<Error>> {
    let imgur_id_re = Regex::new(r"([a-zA-Z0-9_]{5,7})(\.\w+)?$").unwrap();

    let c = is_collection(&url);
    let t = if c { "Album" } else { "Image" };

    // NOTE: captures returns: {<whole regex>, <capture 1>, <capture 2>, ...}
    let id: &str = &imgur_id_re.captures(&url).unwrap()[1];

    let body = match get_data(id, c) {
        Ok(mut d) => d.text()?,
        Err(e) => return Err(Box::new(e))
    };

    // TODO: refactor into a new function ?
    // JSON.parse data
    let json: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e))
    };

    // data[:success] ? data = data[:data] : panic!("!! Scrape failed")
    let mut data;
    if json["success"].as_bool().unwrap() {
        data = json["data"].to_owned();
    } else {
        // should I use eprintln! and process::exit?
        panic!("!! Scrape failed");
    }

    // title = data[:title] || if c { format!("{}_{}", t, id) }
    if data["title"].is_null() && c {
        data["title"] = json!(format!("{}_{}", t, id));
    }

    Ok(data.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_gets_data() {
        assert!(false);
    }

    #[test]
    fn it_returns_true_on_album() {
        let url = "https://imgur.com/a/cdJA3"; // imgur album
        let result = is_collection(url);
        assert!(result);
    }

    #[test]
    fn it_returns_true_on_gallery() {
        let url = "https://imgur.com/gallery/KtiYY";
        let result = is_collection(url);
        assert!(result);
    }

    #[test]
    fn it_returns_false_on_single_image() {
        let url = "https://i.imgur.com/Z94RUOi.jpg";
        let result = is_collection(url);
        assert!(!result);
    }
}
