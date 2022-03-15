enum EventType {
    Click,
    DoubleClick,
    Touch,
    None,
}

struct Event {
    typ: EventType,
}

struct Executor<'a> {
    events: Vec<Event>,
    callback: Box<dyn Fn(&Executor) + 'a>,
}

impl<'a> Executor<'a> {
    fn set_callback(&mut self, cb: impl Fn(&Executor) + 'a) {
        self.callback = Box::new(cb);
    }

    fn process_event(&self) {
        (self.callback)(&self)
    }
}

fn handle_click(exec: &Executor) {
    println!("event len = {}", exec.events.len());
    for event in exec.events.iter() {
        match event.typ {
            EventType::Click => println!("clicked"),
            _ => println!("others"),
        }
    }
    println!("Callback")
}

#[test]
fn test_callback() {
    let mut executor = Executor {
        events: vec![Event {
            typ: EventType::Click
        }],
        callback: Box::new(handle_click),
    };
    executor.process_event();
}