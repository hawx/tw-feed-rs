extern crate hyper;
extern crate oauth_client as oauth;
extern crate rustc_serialize;
extern crate time;

use std::io::BufReader;
use std::io::prelude::*;
use std::str;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use self::hyper::client::Client;
use self::hyper::status::StatusCode;
use self::hyper::header::Authorization;

use self::oauth::Token;

use self::rustc_serialize::{Decodable, Decoder};
use self::rustc_serialize::json;
use self::rustc_serialize::json::DecodeResult;

use self::time::Tm;

const SAMPLE_STREAM: &'static str = "https://stream.twitter.com/1.1/statuses/sample.json";

struct TmWrapper {
    time: Tm
}

impl Decodable for TmWrapper {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let r = try!(d.read_str());
        let f = "%a %b %e %H:%M:%S %z %Y";

        match time::strptime(&r, f) {
            Ok(t) => Ok(TmWrapper { time: t }),
            Err(_) => Err(d.error(&format!("could not parse time: {}", r)))
        }
    }
}

trait JsonObjectStreamer: Sized {
    fn json_objects(&mut self) -> JsonObjects<Self>;
}

impl<T: BufRead> JsonObjectStreamer for T {
    fn json_objects(&mut self) -> JsonObjects<T> {
        JsonObjects { reader: self }
    }
}

struct JsonObjects<'a, B> where B: 'a {
    reader: &'a mut B
}

impl<'a, B> Iterator for JsonObjects<'a, B> where B: BufRead + 'a {

    type Item = Option<Decoded>;

    fn next(&mut self) -> Option<Option<Decoded>> {
        let mut buf: Vec<u8> = Vec::new();

        let _ = self.reader.read_until(b'\r', &mut buf);

        if buf.last() == Some(&b'\r') {
            buf.pop();
            let mut b: String = String::new();
            match self.reader.read_line(&mut b) {
                Ok(_)  => (),
                Err(_) => return None,
            }
        }

        let line = match str::from_utf8(&buf) {
            Ok(line) => line,
            Err(_)   => return None
        };

        let decoded: DecodeResult<Decoded> = json::decode(line);
        match decoded {
            Ok(r) => Some(Some(r)),
            Err(_) => Some(None)
        }
    }
}

#[derive(RustcDecodable)]
struct Decoded {
    id: i64,
    text: String,
    created_at: TmWrapper,
    user: User
}

#[derive(RustcDecodable)]
struct User {
    screen_name: String
}

pub fn create_token<'a>(consumer_key: String, consumer_secret: String) -> Token<'a> {
    Token::new(consumer_key, consumer_secret)
}

pub fn get_timeline(consumer: &Token, access: &Token, tweets: Arc<Mutex<VecDeque<Tweet>>>) {
    let header = oauth::authorization_header("GET", SAMPLE_STREAM, &consumer, Some(access), None);

    let resp = Client::new()
        .get(SAMPLE_STREAM)
        .header(Authorization(header))
        .send();

    if let Ok(res) = resp {
        if res.status != StatusCode::Ok {
            println!("Got status code {}", res.status);
            return
        }

        for obj in BufReader::new(res).json_objects() {
            if let Some(tweet) = obj {
                let mut tweets = tweets.lock().unwrap();
                if tweets.len() > 20 {
                    tweets.pop_front();
                }

                tweets.push_back(Tweet {
                    text: tweet.text,
                    link: format!("https://twitter.com/{}/status/{}", tweet.user.screen_name, tweet.id),
                    created_at: tweet.created_at.time
                });
            }
        }
    }
}

pub struct Tweet {
    pub link: String,
    pub text: String,
    pub created_at: Tm
}
