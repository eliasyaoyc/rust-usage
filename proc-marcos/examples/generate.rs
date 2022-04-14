use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod generated {
    use proc_marcos::generate;
    generate!("/Users/eliasyao/Desktop/rust-usage/proc-marcos/fixtures/person.json");
}

use generated::*;

fn main() {
    let person = Person {
        first_name: "Tyr".into(),
        last_name: "Chen".into(),
        skill: Skill {
            name: "Rust".into(),
        },
    };
    println!("{:#?}", person);
}