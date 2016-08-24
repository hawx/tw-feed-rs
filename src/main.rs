extern crate docopt;
extern crate iron;
extern crate rustc_serialize;
extern crate time;

#[macro_use]
extern crate horrorshow;

use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

use docopt::Docopt;

use iron::prelude::*;
use iron::status;

use horrorshow::prelude::*;

mod twitter;

const USAGE: &'static str = "
Twitter feed.

Usage:
  feed --consumer-key=<key> --consumer-secret=<secret> --access-key=<key> --access-secret=<secret>

Options:
  --consumer-key=<key>          Consumer key
  --consumer-secret=<secret>    Consumer secret
  --access-key=<key>            Access key
  --access-secret=<secret>      Access secret
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_consumer_key: String,
    flag_consumer_secret: String,
    flag_access_key: String,
    flag_access_secret: String
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    let title = "tw-feed";
    let link = "http://example.com";

    let tweets = Arc::new(Mutex::new(VecDeque::new()));
    let writer = tweets.clone();

    let consumer = twitter::create_token(args.flag_consumer_key, args.flag_consumer_secret);
    let access = twitter::create_token(args.flag_access_key, args.flag_access_secret);

    thread::spawn(move || {
        twitter::get_timeline(&consumer, &access, writer);
    });

    let chain = Chain::new(move |_: &mut Request| {
        let tweets = tweets.lock().unwrap();

        let body = html! {
            rss(version="2.0") {
                channel {
                    title { : title }
                    link { : link }
                    pubDate { : format_args!("{}", time::now().rfc822()) }

                    @ for tweet in tweets.iter() {
                        item {
                            link { : tweet.link.to_string() }
                            description { : tweet.text.to_string() }
                            pubDate { : format_args!("{}", tweet.created_at.rfc822()) }
                        }
                    }
                }
            }
        }.into_string().unwrap();

        Ok(Response::with((status::Ok,
                           format!("<!xml version=\"1.0\" encoding=\"utf-8\"?>{}", body))))
    });

    println!("Running on http://localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}
