trait Animal {
    fn name(&self) -> String;
}

trait IntoAnimal {
    type IntoAnimal: Animal;

    fn into_animal(self) -> Self::IntoAnimal;
}

impl<A: Animal> IntoAnimal for A {
    type IntoAnimal = A;

    fn into_animal(self) -> Self::IntoAnimal {
        self
    }
}

struct Apeman;

impl Apeman {
    pub fn new() -> Apeman {
        Apeman
    }
}

impl Animal for Apeman {
    fn name(&self) -> String {
        "猿人".into()
    }
}

struct People;

impl IntoAnimal for People {
    type IntoAnimal = Apeman;

    fn into_animal(self) -> Self::IntoAnimal {
        Apeman::new()
    }
}

#[test]
fn test() {
    let people = People {};
    let animal = people.into_animal();
    println!("{}", animal.name());
}
