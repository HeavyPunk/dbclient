pub struct DummyFetcher {
    objects: Vec<String>,
}

impl DummyFetcher {
    pub fn new() -> Self {
        Self {
            objects: vec![
                String::from("dummy_obj_1"),
                String::from("dummy_obj_2"),
                String::from("dummy_obj_3"),
                String::from("dummy_obj_4"),
            ],
        }
    }
}
