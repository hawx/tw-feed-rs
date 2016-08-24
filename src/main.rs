extern crate docopt;
extern crate iron;
extern crate rustc_serialize;

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

    let tweets = Arc::new(Mutex::new(VecDeque::new()));
    let writer = tweets.clone();

    let consumer = twitter::create_token(args.flag_consumer_key, args.flag_consumer_secret);
    let access = twitter::create_token(args.flag_access_key, args.flag_access_secret);

    thread::spawn(move || {
        twitter::get_timeline(&consumer, &access, writer);
    });

    let chain = Chain::new(move |_: &mut Request| {
        let tweets = tweets.lock().unwrap();

        /*
        <?xml version="1.0" encoding="UTF-8"?><rss version="2.0">
          <channel>
            <title>tw-linkfeed</title>
            <link>http://tw-linkfeed.hawx.me/feed</link>
            <description></description>
            <pubDate>24 Aug 16 13:51 EDT</pubDate>
            <item>
              <title>Italy earthquake: Death toll rises to at least 120 - BBC NewsBBC News</title>
              <link>http://bbc.in/2bMtsoU</link>
              <description>RT @BBCBreaking: Italy earthquake latest:&#xA;- At least 120 people dead &#xA;- Magnitude 6.2&#xA;- Three-quarters of Amatrice town destroyed&#xA;https://tâ€¦</description>
              <pubDate>24 Aug 16 17:31 UTC</pubDate>
            </item>
        */

        let body = html! {
            rss(version="2.0") {
                channel {
                    title { : "tw-linkfeed" }
                    link { : "https://feed.hawx.me/tw-linkfeed" }
                    description { : "" }
                    pubDate { : "" }

                    @ for tweet in tweets.iter() {
                        item {
                            title { : tweet }
                            link { : "" }
                            description { : format_args!("{}", tweet) }
                            pubDate { : "" }
                        }
                    }
                }
            }
        }.into_string().unwrap();

        Ok(Response::with((status::Ok, body)))
    });

    println!("Running on http://localhost:3000");
    Iron::new(chain).http("localhost:3000").unwrap();
}
