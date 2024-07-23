use v8::{Context, FunctionCallbackArguments, Local, ReturnValue, HandleScope};

/// Initializes custom functions in the V8 context.
/// 
/// This function adds custom JavaScript functions to the global object
/// of the V8 context, making them available for use in the executed JavaScript code.
/// 
/// # Arguments
/// * `scope` - A mutable reference to the V8 HandleScope.
/// * `context` - A Local handle to the V8 Context.
pub fn init_functions(scope: &mut v8::HandleScope, context: Local<Context>) {
  let global = context.global(scope);

  let fetch_function = v8::Function::new(scope, fetch).unwrap();
  let fetch_name = v8::String::new(scope, "fetch").unwrap();

  global.set(scope, fetch_name.into(), fetch_function.into()).unwrap();
}

/// Implements the 'fetch' function for JavaScript.
/// 
/// This function is a callback that can be called from JavaScript. It performs
/// an HTTP GET request to the URL provided as an argument and returns the 
/// response as a string.
/// 
/// # Arguments
/// * `scope` - A mutable reference to the V8 HandleScope.
/// * `args` - The arguments passed from JavaScript.
/// * `retval` - The return value to be sent back to JavaScript.
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
