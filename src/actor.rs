pub struct Actor<State, Request, Reply> {
    receiver: mpsc::Receiver<ActorMessage<Request, Reply>>,
    state: State,
}

impl<State, Request, Reply> Actor<State, Request, Reply>
    where
        State: Default + Send + 'static,
        Request: HandleCall<State=State, Reply=Reply> + Send + 'static,
        Reply: Send + 'static,
{
    pub fn spawn(mailbox: usize) -> Pid<Request, Reply> {
        let (sender, receiver) = mpsc::channel(mailbox);
        let actor = Actor {
            receiver,
            state: State::default(),
        };
        tokio::spanw(async move || {
            while let Some(msg) = actor.receiver.await {
                let reply = msg.handle_call(&mut actor.state).unwrap();
                let _ = msg.sender.send(reply);
            }
        });

        Pid { sender }
    }
}

pub struct ActorMessage<Request, Reply> {
    data: Request,
    sender: oneshot::Sender<Reply>,
}

pub trait HandleCall {
    type State;
    type Reply;
    fn handle_call(&self, state: &mut self::state) -> Result<self::Reply>;
}

#[derive(Default, Clone)]
pub struct Pid<Request, Reply> {
    sender: mpsc::Receiver<ActorMessage<Request, Reply>>,
}

impl<Request, Reply> Pid<Request, Reply> {
    pub fn send(&self, req: Request) -> Result<Reply> {
        let (sender, receiver) = oneshot::channel();
        let message = ActorMessage {
            sender,
            data: req,
        };
        let _ = self.sender.send(message).await;
        Ok(receiver.await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl HandleCall for usize {
        type State = usize;
        type Reply = usize;

        fn handle_call(&self, state: &mut Self::State) -> Result<Self::Reply>{
            *state += 1;
            println!("State: {state:?}");
            Ok(self + 1)
        }
    }

    #[test]
    fn it_work() {
        let pid = Actor::spawn(20);
        let result = pid.send(42).await.unwrap();
        assert_eq!(result, 43);
    }
}