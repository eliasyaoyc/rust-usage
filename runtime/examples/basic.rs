use std::sync::mpsc;

use runtime::executor;
use runtime::threadpool::ThreadPool;

fn main() {
    let pool = ThreadPool::new();

    let (tx, rx) = mpsc::channel();
    let fut_values = async {
        let fut_tx_result =
            async move { (0..100).for_each(|v| tx.send(v).expect("Failed to send")) };

        pool.spwan(fut_tx_result);

        let mut pending = vec![];
        while let Ok(v) = rx.recv() {
            pending.push(v);
        }
        pending
    };

    let values: Vec<i32> = executor::block_on(fut_values);

    println!("Values = {values:?}");
}
