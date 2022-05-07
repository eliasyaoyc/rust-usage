use v8_live::{JsRuntime, JsRuntimeParams};

fn main() {
    JsRuntime::init();
    let mut runtime = JsRuntime::new(JsRuntimeParams::default());
    let code = r#"
    function hello(){
       return {
         status: 200,
         message: "Hello, world!"
       };
    }
    hello();
    "#;
    let result = runtime.execute_script(code);
    println!("Result is: {:#?}", result);
}
