use std::{
    collections::HashSet,
    str,
    vec::Vec,
};
use actix_web::http::{StatusCode, header::HeaderMap};
use bytes::Bytes;
use log::debug;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Ngrams {
    n: usize,
    src: Vec<String>,
    set: HashSet<String>,
}

impl Ngrams {
    pub fn new(n: usize, s: String) -> Self {
        let src = s.trim().split_whitespace().map(str::to_string).collect::<Vec<String>>();
        let mut set: HashSet<String> = HashSet::new();
        if src.len() < n {
            set.insert(src[..].to_vec().join(" "));
        } else {
            for i in 0..=(src.len()-n) {
                set.insert(src[i..(i+n)].to_vec().join(" "));
            }
        }
        Self { n, src, set }
    }

    pub fn compatibility_score(&self, other: &Self) -> u32 {
        self.set.intersection(&other.set).count().try_into().unwrap()
    }
}


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MiddlewareDataHttp {
    status: StatusCode,
    headers: HeaderMap,
}

impl MiddlewareDataHttp {
    pub fn new(status: StatusCode, headers: HeaderMap) -> Self {
        Self { status, headers }
    }

    pub fn split(&self) -> (StatusCode, HeaderMap) {
        (self.status.clone(), self.headers.clone())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MiddlewareData {
    request: Ngrams,
    response: Bytes,
    http: Option<MiddlewareDataHttp>
}

impl MiddlewareData {
    pub fn new(req: String, resp: Bytes, http: Option<MiddlewareDataHttp>) -> Self {
        Self {
            request: Ngrams::new(3, req),
            response: resp,
            http,
        }
    }
}


pub type Db = Vec<MiddlewareData>;

pub trait Dbly {
    fn find_best_response(&self, req: String) -> (Bytes, MiddlewareDataHttp);
}

impl Dbly for Db {
    fn find_best_response(&self, req: String) -> (Bytes, MiddlewareDataHttp) {
        let req_ngrams = Ngrams::new(3, req);
        let mut best_score: u32 = 0;
        let mut idx = 0;
        
        for (i, data) in self.iter().enumerate() {
            let new_score = req_ngrams.compatibility_score(&data.request);
            if new_score >= best_score {
                best_score = new_score;
                idx = i;
            }
            debug!("cmp score between '{:?}' and '{:?}' --- {:?}", &req_ngrams.src, &data.request.src, new_score);
        };
        
        (self[idx].response.clone(), self[idx].http.as_ref().expect("have no http stuff").clone())
    }
}