extern crate docopt;
extern crate iron;
extern crate rustc_serialize;

use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

use docopt::Docopt;

use iron::prelude::*;
use iron::status;

mod twitter;

const USAGE: &'static str = "
Twitter feed.

Usage:
  feed --consumer-key=<key> --consumer-secret=<secret>

Options:
  --consumer-key=<key>          Consumer key
  --consumer-secret=<secret>    Consumer secret
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_consumer_key: String,
    flag_consumer_secret: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let access_token = twitter::get_access_token(&args.flag_consumer_key, &args.flag_consumer_secret)
        .unwrap();

    let tweets = Arc::new(Mutex::new(VecDeque::new()));
    let writer = tweets.clone();

    thread::spawn(move || {
        twitter::get_timeline(access_token.to_string(), writer);
    });

    let chain = Chain::new(move |_: &mut Request| {
        let tweets = tweets.lock().unwrap();
        let el = tweets.get(0).unwrap();

        return Ok(Response::with((status::Ok, el.to_string())));
    });

    Iron::new(chain).http("localhost:3000").unwrap();
}
