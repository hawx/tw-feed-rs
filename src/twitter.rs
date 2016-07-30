extern crate rustc_serialize;
extern crate hyper;

use std::io::prelude::*;

use self::hyper::client::Client;
use self::hyper::status::StatusCode;
use self::hyper::header::{Authorization, ContentType};
use self::hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use self::rustc_serialize::base64::{ToBase64, STANDARD};
use self::rustc_serialize::json::Json;

pub fn get_access_token<'a>(consumer_key: &'a String, consumer_secret: &'a String) -> Option<String> {
    let bearer_token_credentials = (format!("{}:{}", consumer_key, consumer_secret)).as_bytes().to_base64(STANDARD);

    let mut res = Client::new()
        .post("https://api.twitter.com/oauth2/token")
        .body("grant_type=client_credentials")
        .header(Authorization(format!("Basic {}", bearer_token_credentials)))
        .header(ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded,
                                 vec![(Attr::Charset, Value::Utf8)])))
        .send()
        .unwrap();

    if res.status != StatusCode::Ok {
        return None
    }

    let mut buffer = String::new();
    let _ = res.read_to_string(&mut buffer);

    if let Ok(data) = Json::from_str(&buffer) {
        if let Some(obj) = data.as_object() {
            if let Some(token_type) = obj.get("token_type").and_then(|x| x.as_string())  {
                if token_type == "bearer" {
                    if let Some(access_token) = obj.get("access_token").and_then(|x| x.as_string()) {
                        return Some(access_token.to_owned());
                    }
                }
            }
        }
    }

    return None
}

pub fn get_timeline(access_token: String) {
    let mut res = Client::new()
        .get("https://userstream.twitter.com/1.1/user.json?with=user")
        .header(Authorization(format!("Bearer {}", access_token)))
        .send()
        .unwrap();

    if res.status != StatusCode::Ok {
        return
    }


}
