
use v8::{Context, FunctionCallbackArguments, Local, ReturnValue, HandleScope};

pub fn init_functions(scope: &mut v8::HandleScope, context: Local<Context>) {
  let global = context.global(scope);

  let fetch_function = v8::Function::new(scope, fetch).unwrap();
  let fetch_name = v8::String::new(scope, "fetch").unwrap();

  global.set(scope, fetch_name.into(), fetch_function.into()).unwrap();
}

fn fetch(
  scope: &mut HandleScope,
  args: FunctionCallbackArguments,
  mut retval: ReturnValue
) {
  let url = args.get(0).to_string(scope)
    .unwrap()
    .to_rust_string_lossy(scope);
  
  match ureq::get(&url).call() {
    Ok(response) => {
      match response.into_string() {
        Ok(text) => {
          let v8_string = v8::String::new(scope, &text).unwrap();
          retval.set(v8_string.into());
        },
        Err(_) => {
          retval.set_undefined();
        }
      }
    },
    Err(_) => {
      retval.set_undefined();
    }
  }
}
