use axum::Json;
use serde::Deserialize;
use axum::extract::Query;
use serde_json::{json, Value};

use crate::{runner, TIMEOUT};
use crate::utils::empty_string_as_none;

#[derive(Debug, Deserialize)]
pub struct FunctionParams {
	#[serde(default, deserialize_with = "empty_string_as_none")]
	args: Option<String>
}

pub async fn runner(Query(params): Query<FunctionParams>) -> Json<Value> {
	let code = r#"
    async function test(size) {
      let str = '';
      for (let i = 0; i < size; i++) {
        str += 'a';  // Añadimos un carácter para cada iteración
      }
      return str;
    }

		test
	"#.to_string();

	let args = params.args.unwrap_or("{}".to_string());

	match runner::run(code, args, TIMEOUT) {
		Ok(value) => Json(json!(value)),
		Err(error) => Json(json!({ "error": error }))
	}
}