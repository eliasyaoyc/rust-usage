use std::sync::{Mutex, Arc};
use std::task::{Waker, Context, Poll};
use std::pin::Pin;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use futures::future::BoxFuture;
use std::future::Future;

struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();

            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });
        TimerFuture {
            shared_state
        }
    }
}

pub mod scope {
    use {
        futures::{
            future::{BoxFuture, FutureExt},
            task::{waker_ref, ArcWake},
        },
        std::{
            future::Future,
            sync::mpsc::{sync_channel, Receiver, SyncSender},
            sync::{Arc, Mutex},
            task::{Context, Poll},
            time::Duration,
        },
    };

    struct Executor {
        ready_queue: Receiver<Arc<Task>>,
    }

    #[derive(Clone)]
    struct Spawner {
        task_sender: SyncSender<Arc<Task>>,
    }

    struct Task {
        future: Mutex<Option<BoxFuture<'static, ()>>>,
        task_sender: SyncSender<Arc<Task>>,
    }

    fn new_executor_and_spawner() -> (Executor, Spawner) {
        const MAX_QUEUE_TASKS: usize = 10_000;
        let (task_sender, ready_queue) = sync_channel(MAX_QUEUE_TASKS);
        (Executor { ready_queue }, Spawner { task_sender })
    }

    impl Spawner {
        fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
            let future = future.boxed();
            let task = Arc::new(Task {
                future: Mutex::new(Some(future)),
                task_sender: self.task_sender.clone(),
            });
            self.task_sender.send(task).expect("too many tasks queued");
        }
    }

    impl ArcWake for Task {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            let cloned = arc_self.clone();
            arc_self.task_sender.send(cloned).expect("too many tasks queued.")
        }
    }

    impl Executor {
        fn run(&self) {
            while let Ok(task) = self.ready_queue.recv() {
                let mut future_slot = task.future.lock().unwrap();
                if let Some(mut future) = future_slot.take() {
                    let waker = waker_ref(&task);
                    let context = &mut Context::from_waker(&*waker);
                    if future.as_mut().poll(context).is_pending() {
                        *future_slot = Some(future)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::timer_future::TimerFuture;
    use std::time::Duration;

    #[test]
    fn t() {
        fn main() {
            let (executor, spawner) = new_executor_and_spawner();

            // Spawn a task to print before and after waiting on a timer.
            spawner.spawn(async {
                println!("howdy!");
                // Wait for our timer future to complete after two seconds.
                TimerFuture::new(Duration::new(2, 0)).await;
                println!("done!");
            });

            // Drop the spawner so that our executor knows it is finished and won't
            // receive more incoming tasks to run.
            drop(spawner);

            // Run the executor until the task queue is empty.
            // This will print "howdy!", pause, and then print "done!".
            executor.run();
        }
    }
}