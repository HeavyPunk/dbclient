use super::fetcher::{FetchResult, Fetcher};

pub struct DummyFetcher {
    objects: Vec<String>
}

impl DummyFetcher {
    pub fn new() -> Self {
        Self {
            objects: vec![String::from("dummy_obj_1"), String::from("dummy_obj_2"), String::from("dummy_obj_3"), String::from("dummy_obj_4")]
        }
    }
}

impl Fetcher for DummyFetcher {
    fn fetch(&mut self, request: &super::fetcher::FetchRequest) -> Result<super::fetcher::FetchResult, super::fetcher::FetcherError> {
        Ok(FetchResult::multiple(&self.objects))
    }

    fn fetch_db_objects(&mut self) -> Result<FetchResult, super::fetcher::FetcherError> {
        Ok(FetchResult::multiple(&self.objects))
    }
}

