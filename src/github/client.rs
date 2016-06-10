// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;
use std::u32;

use chrono::{DateTime, UTC};
use hyper;
use hyper::client::{RedirectPolicy, Response};
use hyper::header::Headers;
use serde::Deserialize;
use serde_json;

use config::CONFIG;
use error::{DashError, DashResult};
use github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson, PullRequestUrls};

pub const BASE_URL: &'static str = "https://api.github.com";

pub const DELAY: u64 = 300;

type ParameterMap = BTreeMap<&'static str, String>;

header! { (Auth, "Authorization") => [String] }
header! { (UA, "User-Agent") => [String] }
header! { (TZ, "Time-Zone") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (RateLimitRemaining, "X-RateLimit-Remaining") => [u32] }
header! { (RateLimitReset, "X-RateLimit-Reset") => [i64] }
header! { (Link, "Link") => [String] }

const PER_PAGE: u32 = 100;

#[derive(Debug)]
pub struct Client {
    token: String,
    ua: String,
    client: hyper::Client,
    rate_limit: u32,
    rate_limit_timeout: DateTime<UTC>,
}

impl Client {
    pub fn new() -> Self {
        let mut client = hyper::Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowAll);

        Client {
            token: CONFIG.github_access_token.clone(),
            ua: CONFIG.github_user_agent.clone(),
            client: client,
            rate_limit: u32::MAX,
            rate_limit_timeout: UTC::now(),
        }
    }

    pub fn issues_since(&self, repo: &str, start: DateTime<UTC>) -> DashResult<Vec<IssueFromJson>> {

        let url = format!("{}/repos/{}/issues", BASE_URL, repo);
        let mut params = ParameterMap::new();

        params.insert("state", "all".to_string());
        params.insert("since", format!("{:?}", start));
        params.insert("state", "all".to_string());
        params.insert("per_page", format!("{}", PER_PAGE));
        params.insert("direction", "asc".to_string());

        // make the request
        self.models_since(&url, &params)
    }

    pub fn comments_since(&self,
                          repo: &str,
                          start: DateTime<UTC>)
                          -> DashResult<Vec<CommentFromJson>> {
        let url = format!("{}/repos/{}/issues/comments", BASE_URL, repo);
        let mut params = ParameterMap::new();

        params.insert("sort", "created".to_string());
        params.insert("direction", "asc".to_string());
        params.insert("since", format!("{:?}", start));
        params.insert("per_page", format!("{}", PER_PAGE));

        self.models_since(&url, &params)
    }

    fn models_since<M: Deserialize>(&self,
                                    start_url: &str,
                                    params: &ParameterMap)
                                    -> DashResult<Vec<M>> {
        let mut res = try!(self.request(start_url, Some(&params)));

        // let's try deserializing!
        let mut buf = String::new();
        try!(res.read_to_string(&mut buf));

        let mut models = try!(serde_json::from_str::<Vec<M>>(&buf));

        let mut next_url = Self::next_page(&res.headers);
        while next_url.is_some() {
            // TODO figure out a better rate limit
            sleep(Duration::from_millis(DELAY));
            let url = next_url.unwrap();
            let mut next_res = try!(self.request(&url, None));

            buf.clear();
            try!(next_res.read_to_string(&mut buf));

            models.extend(try!(serde_json::from_str::<Vec<M>>(&buf)));

            next_url = Self::next_page(&next_res.headers);
        }

        Ok(models)
    }

    pub fn fetch_pull_request(&self, pr_info: &PullRequestUrls) -> DashResult<PullRequestFromJson> {
        let url = pr_info.get("url");

        if let Some(url) = url {
            let mut res = try!(self.request(url, None));
            let mut buf = String::new();
            try!(res.read_to_string(&mut buf));

            Ok(try!(serde_json::from_str::<PullRequestFromJson>(&buf)))
        } else {
            Err(DashError::Misc)
        }
    }

    fn next_page(h: &Headers) -> Option<String> {
        if let Some(lh) = h.get::<Link>() {
            for link in (**lh).split(",").map(|s| s.trim()) {

                let tokens = link.split(";").map(|s| s.trim()).collect::<Vec<_>>();

                if tokens.len() != 2 {
                    continue;
                }

                if tokens[1] == "rel=\"next\"" {
                    let url = tokens[0].trim_left_matches('<').trim_right_matches('>').to_string();
                    return Some(url);
                }
            }
        }

        None
    }

    fn request<'a>(&self,
                   url: &'a str,
                   params: Option<&ParameterMap>)
                   -> Result<Response, hyper::error::Error> {

        let qp_string = match params {
            Some(p) => {
                let mut qp = String::from("?");
                for (k, v) in p.iter() {
                    if qp.len() > 1 {
                        qp.push('&');
                    }
                    qp.push_str(&format!("{}={}", k, v));
                }
                qp
            }
            None => "".to_string(),
        };

        let url = format!("{}{}", url, qp_string);

        debug!("GETing: {}", &url);

        self.client
            .get(&url)
            .header(Auth(format!("token {}", &self.token)))
            .header(UA(self.ua.clone()))
            .header(TZ("UTC".to_string()))
            .header(Accept("application/vnd.github.v3".to_string()))
            .header(hyper::header::Connection::close())
            .send()
    }
}
