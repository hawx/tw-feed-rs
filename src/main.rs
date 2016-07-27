extern crate docopt;
extern crate iron;
extern crate rustc_serialize;

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

    Iron::new(move |_: &mut Request| {
        let body = twitter::get_access_token(&args.flag_consumer_key, &args.flag_consumer_secret);

        if let Some(b) = body {
            return Ok(Response::with((status::Ok, b.to_string())));
        } else {
            return Ok(Response::with((status::InternalServerError)));
        }
    }).http("localhost:3000").unwrap();
}
