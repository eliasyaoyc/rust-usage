use anyhow::Result;
use wasmtime::*;

fn main() -> Result<()> {
    let engine = Engine::default();
    let wat = r#"
         (module
              (import "host" "hello" (func $host_hello (param i32)))

              (func (export "hello")
                   i32.const 3
                   call $host_hello
                                   )
         )
    "#;

    let gcd_wat = r#"
    (module
  (func $gcd (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        br_if 0 (;@2;)
        local.get 1
        local.set 2
        br 1 (;@1;)
      end
      loop  ;; label = @2
        local.get 1
        local.get 0
        local.tee 2
        i32.rem_u
        local.set 0
        local.get 2
        local.set 1
        local.get 0
        br_if 0 (;@2;)
      end
    end
    local.get 2
  )
  (export "gcd" (func $gcd))
)

    "#;
    let module = Module::new(&engine, wat)?;
    let mut store = Store::new(&engine, 4);
    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, u32>, param: i32| {
        println!("Got {} from WebAssembly", param);
        println!("my host state is: {}", caller.data());
    });

    let instance = Instance::new(&mut store, &module, &[host_hello.into()])?;
    let hello = instance.get_typed_func::<(), (), _>(&mut store, "hello")?;
    hello.call(&mut store, ())?;

    let mut store = Store::<()>::default();
    let module = Module::new(store.engine(),gcd_wat)?;
    let instance = Instance::new(&mut store, &module, &[])?;

    let gcd = instance.get_typed_func::<(i32,i32),i32,_>(&mut store, "gcd")?;
    println!("gcd(6,27) = {}", gcd.call(&mut store,(6,27))?);
    Ok(())
}
