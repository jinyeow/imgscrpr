use regex::Regex;
use reqwest;
use scraper::{Html, Selector};
use serde_json::Value;

use std::error::Error;

// Examples:
//  https://gfycat.com/FancyMerryBufflehead
//  https://thumbs.gfycat.com/FancyMerryBufflehead-size_restricted.gif
//  https://thumbs.gfycat.com/FancyMerryBufflehead-max-14mb.gif
//  https://giant.gfycat.com/DevotedWavyDodo.gif

// NOTE:
// if gif is too small or not high enough quality it uses the 'thumbs' version
// else it uses 'giant'
// it will return a 403 if it is not under 'giant'

fn get_value(fragment: &Html, selector: &str, attr: &str) -> Result<String, ()> {
    let s = Selector::parse(selector)?;

    let result = fragment.select(&s)
        .next().unwrap()
        .value()
        .attr(attr).unwrap();

    Ok(result.to_string())
}

// TODO: turn all the unwrap()s into Box::new(Error)
pub fn scrape_data(url: &str) -> Result<Value, Box<Error>> {
    let id = Regex::new(r"(\w+)$").unwrap()
        .captures(url).unwrap()
        .get(1).unwrap()
        .as_str();

    let mut res = reqwest::get(url).unwrap();
    assert!(res.status().is_success());

    let body = String::from(res.text().unwrap());

    let fragment  = Html::parse_document(&body);
    let webm      = get_value(&fragment, "source#webmSource", "src").unwrap();
    let mp4       = get_value(&fragment, "source#mp4Source", "src").unwrap();
    let large_gif = get_value(&fragment, "a#large-gif", "href").unwrap();
    let small_gif = get_value(&fragment, "a#small-gif", "href").unwrap();

    let data = json!({
        "images": [
            { "link": webm,      "id": id, "ext": "webm" },
            { "link": mp4,       "id": id, "ext": "mp4" },
            { "link": large_gif, "id": id, "ext": "gif" },
            { "link": small_gif, "id": format!("{}-size_restricted", id), "ext": "gif" },
        ]
    });

    Ok(data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_returns_the_selected_value_given_valid_selector_and_attribute() {
        let url = "https://gfycat.com/FancyMerryBufflehead";
        let mut res = reqwest::get(url).unwrap();
        assert!(res.status().is_success());

        let body = String::from(res.text().unwrap());
        let fragment = Html::parse_document(&body);

        let webm      = get_value(&fragment, "source#webmSource", "src").unwrap();
        assert_eq!(webm, "https://giant.gfycat.com/FancyMerryBufflehead.webm");

        let mp4       = get_value(&fragment, "source#mp4Source", "src").unwrap();
        assert_eq!(mp4, "https://giant.gfycat.com/FancyMerryBufflehead.mp4");

        let large_gif = get_value(&fragment, "a#large-gif", "href").unwrap();
        assert_eq!(large_gif,
                   "https://thumbs.gfycat.com/FancyMerryBufflehead-max-14mb.gif");

        let small_gif = get_value(&fragment, "a#small-gif", "href").unwrap();
        assert_eq!(small_gif,
                   "https://thumbs.gfycat.com/FancyMerryBufflehead-size_restricted.gif");
    }
}
