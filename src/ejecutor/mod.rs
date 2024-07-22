mod functions;

use std::ffi::c_void;

use functions::init_functions;
use v8::{ContextScope, HandleScope, IsolateHandle, Local, Script, TryCatch, Value};

use crate::{HEAP_LIMITS, POOL_SIZE};

pub fn init_ejecutor() {
  let platform = v8::new_default_platform(POOL_SIZE, false)
    .make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
}

pub fn run(code: String, args: String, timeout_ms: u64) -> Result<String, String> {
  let duration = std::time::Duration::from_millis(timeout_ms);
	let (send , rcv) = std::sync::mpsc::channel();

	std::thread::spawn(move || {
		send.send(run_function(code.as_str(), args.as_str()))
	});

	match rcv.recv_timeout(duration) {
		Ok(result) => {
      match result {
        Ok(result) => Ok(result),
        Err(e) => Err(e)
      }
    },
		Err(_) => Err("Timeout error".to_string())
	}
}

pub fn run_function(code: &str, args: &str) -> Result<String, String> {
  let (init_heap_limit, max_heap_limit) = HEAP_LIMITS;
  let params = v8::CreateParams::default()
    .heap_limits(init_heap_limit, max_heap_limit);

  let isolate = &mut v8::Isolate::new(params);
  let handle = isolate.thread_safe_handle();

  let heap_context = Box::new(HeapContext {
    handle: handle.clone(),
  });
  let heap_ctx_ptr = Box::into_raw(heap_context);

  isolate.add_near_heap_limit_callback(
    near_heap_limit_callback, 
    heap_ctx_ptr as *mut c_void
  );

  let handle_scope = &mut v8::HandleScope::new(isolate);
  let context = v8::Context::new(handle_scope);
  let scope = &mut v8::ContextScope::new(handle_scope, context);

  init_functions(scope, context);

  let func = create_function(scope, code)?;
  
  let args = v8::String::new(scope, args).unwrap();
  let args = match v8::json::parse(scope, args) {
    Some(args) => args,
    None => return Err("Parametros incompatibles".to_string())
  };

  execute_function(scope, func, &[args])
}

pub fn create_function<'a>(
  scope: &mut ContextScope<'a, HandleScope>,
  code: &str,
) -> Result<Local<'a, v8::Function>, String> {
  let mut try_catch = TryCatch::new(scope);
  let source = v8::String::new(&mut try_catch, code).unwrap();

  let script = Script::compile(&mut try_catch, source, None)
    .ok_or_else(|| format!("Error al compilar el script: {:?}", try_catch.exception()))?;

  let result = script.run(&mut try_catch)
    .ok_or_else(|| format!("Error al ejecutar el script: {:?}", try_catch.exception()))?;

  v8::Local::<v8::Function>::try_from(result)
    .map_err(|e| format!("El resultado no es una función: {:?}", e))
}

pub fn execute_function<'a>(
  scope: &mut ContextScope<'a, HandleScope>,
  func: Local<'a, v8::Function>,
  args: &[Local<'a, Value>],
) -> Result<String, String> {
  let mut try_catch = TryCatch::new(scope);
  let global = try_catch.get_current_context().global(&mut try_catch);

  let promise = func.call(&mut try_catch, global.into(), args)
    .ok_or_else(|| format!("Error al ejecutar la función: {:?}", try_catch.exception()))?;

  let promise = v8::Local::<v8::Promise>::try_from(promise)
    .map_err(|e| format!("El resultado no es una promesa: {:?}", e))?;

  while promise.state() == v8::PromiseState::Pending {
    let _ = &mut try_catch.perform_microtask_checkpoint();
  }

  match promise.state() {
    v8::PromiseState::Fulfilled => {
      let result = promise.result(&mut try_catch);

      if result.is_object() {
        let object = v8::json::stringify(&mut try_catch, result).unwrap();
        Ok(object.to_rust_string_lossy(&mut try_catch))
      } else {
        Ok(result.to_rust_string_lossy(&mut try_catch))
      }
    }
    v8::PromiseState::Rejected => Err(format!(
      "Promesa rechazada: {:?}",
      promise.result(&mut try_catch).to_rust_string_lossy(&mut try_catch)
    )),
    v8::PromiseState::Pending => unreachable!(),
  }
}

struct HeapContext {
  handle: IsolateHandle,
}

extern "C" fn near_heap_limit_callback(
  data: *mut c_void,
  current_heap_limit: usize,
  _initial_heap_limit: usize,
) -> usize {
  let heap_ctx = unsafe { &mut *(data as *mut HeapContext) };
  let _ = heap_ctx.handle.terminate_execution();

  current_heap_limit * 2
}
