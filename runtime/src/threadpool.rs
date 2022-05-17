use std::future::Future;

pub struct ThreadPool {}

impl ThreadPool {
    pub fn new() -> ThreadPool {
        Self {}
    }

    pub fn spwan<F: Future>(&self, f: F) {
        todo!()
    }
}
