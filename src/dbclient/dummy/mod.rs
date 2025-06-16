use super::fetcher::{FetchResult, Fetcher};

pub struct DummyFetcher {
}

impl Fetcher for DummyFetcher {
    fn fetch(&mut self, request: &super::fetcher::FetchRequest) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        Ok(FetchResult::multiple(&vec!["1", "2", "3"]))
    }

    fn fetch_db_objects(&mut self) -> Result<FetchResult, super::fetcher::FetcherError> {
        Ok(FetchResult::multiple(&vec!["dummy_obj_1", "dummy_obj_2", "dummy_obj_3", "dummy_obj_4"]))
    }
}

