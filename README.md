# 1. Type Convert
> 要把一个 struct 转换成另外一个 struct,参考 Iterator 的方式来实现
> Take one struct convert another struct, reference from `Iterator` implementation.
```rust
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
```