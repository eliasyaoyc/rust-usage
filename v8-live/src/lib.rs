mod extension;
mod state;
mod utils;

use once_cell::sync::OnceCell;
use v8::{CreateParams, HandleScope, Isolate, OwnedIsolate, Script, TryCatch, V8};

use crate::extension::Extensions;
use crate::state::JsRuntimeState;
use crate::utils::execute_script;

type LocalValue<'s> = v8::Local<'s, v8::Value>;

pub struct JsRuntime {
    isolate: OwnedIsolate,
}

#[derive(Default)]
pub struct JsRuntimeParams(CreateParams);

impl JsRuntimeParams {
    pub fn new(snapshot: Option<Vec<u8>>) -> Self {
        JsRuntimeParams(CreateParams::default())
    }

    pub fn into_inner(self) -> CreateParams {
        self.0
    }
}

impl JsRuntime {
    pub fn init() {
        static V8_INSTANCE: OnceCell<()> = OnceCell::new();
        V8_INSTANCE.get_or_init(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            V8::initialize_platform(platform);
            V8::initialize();
        });
    }

    pub fn new(params: JsRuntimeParams) -> Self {
        let isolate = Isolate::new(params.into_inner());
        JsRuntime::init_isolate(isolate)
    }

    pub fn execute_script(
        &mut self,
        code: impl AsRef<str>,
    ) -> Result<serde_json::Value, serde_json::Value> {
        let context = JsRuntimeState::get_context(&mut self.isolate);
        let handle_scope = &mut HandleScope::with_context(&mut self.isolate, context);
        match execute_script(handle_scope, code) {
            Ok(value) => Ok(serde_v8::from_v8(handle_scope, value).unwrap()),
            Err(e) => Err(serde_v8::from_v8(handle_scope, e).unwrap()),
        }
    }

    pub fn create_snapshot() -> Vec<u8> {
        todo!()
    }

    pub fn init_isolate(mut isolate: OwnedIsolate) -> Self {
        let state = JsRuntimeState::new(&mut isolate);
        isolate.set_slot(state);
        {
            let context = JsRuntimeState::get_context(&mut isolate);
            let scope = &mut HandleScope::with_context(&mut isolate, context);
            Extensions::install(scope);
        };
        JsRuntime { isolate }
    }
}
