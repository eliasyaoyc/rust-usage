use std::fs;
use std::rc::Rc;

use deno_core::anyhow::Result;
use deno_core::serde::de::DeserializeOwned;
use deno_core::{
    resolve_url_or_path, serde_v8, v8, FsModuleLoader, JsRuntime, RuntimeOptions, Snapshot,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref SNAPSHOT: &'static [u8] = {
        let data = include_bytes!("../snapshots/main.bin");
        let decompressed = zstd::decode_all(&data[..]).unwrap().into_boxed_slice();
        Box::leak(decompressed)
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = RuntimeOptions {
        module_loader: Some(Rc::new(FsModuleLoader)),
        startup_snapshot: Some(Snapshot::Static(&*SNAPSHOT)),
        ..Default::default()
    };
    let mut rt = JsRuntime::new(options);

    let path = format!("{}/examples/basic_module.js", env!("CARGO_MANIFEST_DIR"));

    execute_main_mode(&mut rt, &path).await?;

    // rt.mod_evaluate(id).await??;
    // rt.run_event_loop(false).await?;

    // let code = include_str!("basic.js");
    // let result: String = eval(&mut rt, code).await?;
    // println!("Rust: {result:?}");
    Ok(())
}

#[allow(dead_code)]
async fn eval<T>(rt: &mut JsRuntime, code: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let ret = rt.execute_script("<anon>", code)?;
    let result = rt.resolve_value(ret).await?;
    let scope = &mut rt.handle_scope();
    let result = v8::Local::new(scope, result);
    Ok(serde_v8::from_v8(scope, result)?)
}

#[allow(dead_code)]
async fn execute_main_mode(rt: &mut JsRuntime, path: impl AsRef<str>) -> Result<()> {
    let url = resolve_url_or_path(path.as_ref())?;
    let id = rt.load_main_module(&url, None).await?;
    let mut receiver = rt.mod_evaluate(id);
    tokio::select! {
        resolved = &mut receiver => {
            resolved.expect("failed to evaluate module.")
        }
        _ = rt.run_event_loop(false) => {
            receiver.await.expect("failed to evalutate module.")
        }
    }
}
