use reqwest;
use reqwest::{Response,Error};
use reqwest::header::Authorization;

const API_VERSION: i32 = 3;
const CLIENT_ID: &str = "b5f97137be15df2";

pub fn get_data(id: &str, collection: bool) -> Result<Response, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_gets_data() {
        assert!(false);
    }
}
