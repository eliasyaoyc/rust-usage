use std::ops::Deref;

struct Animal;

impl Animal {
    pub fn new() -> Option<&'static Self> {
        Some(&Animal)
    }
    pub fn name(&self) -> &str {
        "动物"
    }
}

struct Monkey;

impl Deref for Monkey {
    type Target = Animal;

    fn deref(&self) -> &Self::Target {
        Animal::new().unwrap()
    }
}
#[test]
fn test() {
    let u = Monkey;
    println!("{}", u.name())
}
