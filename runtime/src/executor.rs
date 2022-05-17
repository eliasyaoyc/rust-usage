use std::future::Future;

pub fn block_on<F: Future>(f: F) -> F::Output {
    todo!()
}
