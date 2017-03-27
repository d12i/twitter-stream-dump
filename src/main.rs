extern crate hyper;
extern crate oauthcli;
extern crate url;

mod credential;

use hyper::client::{Client, Response};
use hyper::header::{Authorization, Scheme};
use hyper::status::StatusCode;
use oauthcli::{
    OAuthAuthorizationHeader,
    OAuthAuthorizationHeaderBuilder,
    ParseOAuthAuthorizationHeaderError,
    SignatureMethod,
};
use std::env;
use std::fmt::{self, Formatter};
use std::fs::File;
use std::io;
use std::str::FromStr;
use std::thread::{self, JoinHandle};
use url::Url;

/// The endpoint and request parameters.
const REQUEST_URI: &str = "https://userstream.twitter.com/1.1/user.json";

fn main() {
    let n = env::args().skip(1).next().map_or(2, |s| s.parse().unwrap());

    let handles: Vec<_> = (0..n).map(|i| {
        let path = format!("target/stream_{}.dat", i);
        println!("dumping into `{}`...", path);
        dump_into(&path)
    }).collect();

    for jh in handles {
        jh.join().unwrap();
    }
}

/// Connects to a Streaming API endpoint and dumps the raw response body into a file.
fn dump_into(filename: &str) -> JoinHandle<()> {
    let mut res = listen();
    let mut f = File::create(filename).unwrap();

    thread::spawn(move || {
        io::copy(&mut res, &mut f).unwrap();
    })
}

/// Connects to a Streaming API endpoint.
fn listen() -> Response {
    let url = Url::parse(REQUEST_URI).unwrap();

    let oauth = OAuthAuthorizationHeaderBuilder::new(
        "GET", &url, credential::CONSUMER_KEY, credential::CONSUMER_SECRET, SignatureMethod::HmacSha1
    )
        .token(credential::ACCESS_TOKEN, credential::ACCESS_TOKEN_SECRET)
        .finish_for_twitter();

    let res = Client::new()
        .get(url)
        .header(Authorization(OAuth(oauth)))
        .send().unwrap();

    if res.status != StatusCode::Ok {
        panic!("HTTP error: {}", res.status);
    }

    res
}

/// Workaround for azyobuzin/rust-oauthcli#7
#[derive(Clone, Debug)]
struct OAuth(OAuthAuthorizationHeader);

impl FromStr for OAuth {
    type Err = ParseOAuthAuthorizationHeaderError;

    fn from_str(s: &str) -> Result<Self, ParseOAuthAuthorizationHeaderError> {
        Ok(OAuth(OAuthAuthorizationHeader::from_str(s)?))
    }
}

impl Scheme for OAuth {
    fn scheme() -> Option<&'static str> {
        Some("OAuth")
    }

    fn fmt_scheme(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.0.auth_param())
    }
}
