use crate::dbclient::fetcher::FetchResult;

#[derive(Debug)]
pub enum Error {
    NoMessages
}

pub enum Payload {
    DbObjects(FetchResult),
    UserMode(String),
}

pub struct Pipe {
    dbobjects: Vec<FetchResult>,
    user_modes: Vec<String>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            dbobjects: vec![],
            user_modes: vec![],
        }
    }

    pub fn push_message(&mut self, payload: Payload) -> Result<(), Error> {
        match payload {
            Payload::DbObjects(fetch_result) => self.dbobjects.push(fetch_result),
            Payload::UserMode(mode) => self.user_modes.push(mode),
        };
        Ok(())
    }

    pub fn try_get_db_objects(&mut self) -> Result<FetchResult, Error> {
        match self.dbobjects.pop() {
            Some(fetch_result) => Ok(fetch_result),
            None => Err(Error::NoMessages),
        }
    }

    pub fn try_get_user_mode(&mut self) -> Result<String, Error> {
        match self.user_modes.pop() {
            Some(mode) => Ok(mode),
            None => Err(Error::NoMessages),
        }
    }
}

