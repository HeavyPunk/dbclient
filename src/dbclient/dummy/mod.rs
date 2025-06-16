use super::fetcher::{FetchResult, Fetcher};

pub struct DummyFetcher {
}

impl Fetcher for DummyFetcher {
    fn fetch(&mut self, request: &super::fetcher::FetchRequest) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        Ok(FetchResult::multiple(&vec!["1", "2", "3"]))
    }
}

