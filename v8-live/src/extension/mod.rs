use v8::{FunctionCallbackArguments, HandleScope, ReturnValue};

pub struct Extensions;

impl Extensions {
    pub fn install(scope: &mut HandleScope) {
        let bindings = v8::Object::new(scope);
        let name = v8::String::new(scope, "print").unwrap();
        let func = v8::Function::new(scope, print).unwrap();
        bindings.set(scope, name.into(), func.into()).unwrap();
    }
}

fn print(scope: &mut HandleScope, args: FunctionCallbackArguments, mut rv: ReturnValue) {
    let result: serde_json::Value = serde_v8::from_v8(scope, args.get(0)).unwrap();
    println!("Rust says: {result:#?}");
    rv.set(serde_v8::to_v8(scope, result).unwrap());
}
