use bytes::Bytes;
use log::{debug, info};
use std::{
    sync::{Arc, Mutex},
    vec::Vec,
};

use crate::ngrams::{Db, MiddlewareData, Dbly, MiddlewareDataHttp};


#[derive(Debug, Clone)]
pub enum State {
    Record,
    Replay,
}


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UnsafeAppGuts {
    db: Db,
    state: State,
}

impl UnsafeAppGuts {
    pub fn new() -> Self {
        Self {
            db: Vec::new(),
            state: State::Record,
        }
    }

    pub fn insert_data(&mut self, req: String, resp: Bytes, http: Option<MiddlewareDataHttp>) {
        self.db.push(MiddlewareData::new(req, resp, http));

        debug!("Added MiddlewareData to Db: {:?}", self.db[self.db.len()-1]);
    }

    pub fn show_data(&self) {
        info!("{:?}", &self.db)
    }

    pub fn find_best_answer(&self, req: String) -> (Bytes, MiddlewareDataHttp) {
        self.db.find_best_response(req)
    }

    pub fn is_record_state(&self) -> bool {
        match self.state {
            State::Record => true,
            State::Replay => false,
        }
    }

    pub fn change_state(&mut self) {
        self.state = match self.state {
            State::Record => State::Replay,
            State::Replay => State::Record,
        };
        info!("App state changed! Now: {:?}", &self.state);
    }

}

pub type AppGuts = Arc<Mutex<UnsafeAppGuts>>;


