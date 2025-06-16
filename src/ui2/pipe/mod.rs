use crate::dbclient::fetcher::FetchResult;

pub enum Error {
    NoMessages
}

pub enum Payload {
    DbObjects(FetchResult),
}

pub struct Pipe {
    dbobjects: Vec<FetchResult>
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            dbobjects: vec![]
        }
    }

    pub fn push_message(&mut self, payload: Payload) -> Result<(), Error> {
        match payload {
            Payload::DbObjects(fetch_result) => self.dbobjects.push(fetch_result),
        };
        Ok(())
    }

    pub fn try_get_db_objects(&mut self) -> Result<FetchResult, Error> {
        match self.dbobjects.pop() {
            Some(fetch_result) => Ok(fetch_result),
            None => Err(Error::NoMessages),
        }
    }
}

