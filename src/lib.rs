extern crate reqwest;

pub mod imgscrpr {
    // TODO
}

mod imgur {
    use reqwest;
    // use std::io::Read;

    const API_VERSION: i32 = 3;
    const CLIENT_ID: &str = "b5f97137be15df2";

    pub fn get_data(id: &str, collection: bool) -> Result<reqwest::Response, reqwest::Error> {
        let t = if collection {
            "album"
        } else {
            "image"
        };
        let api_url = format!("https//api.imgur.com/{}/{}/{}", CLIENT_ID, t, id);
        reqwest::get(&api_url)
    }

}
