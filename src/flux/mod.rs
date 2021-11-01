use std::str;
use std::process;
use serde::Deserialize;
use serde_json::{ json, from_str };
use std::collections::HashMap;

mod raw;

pub fn get_used_gas() -> i64 {
    unsafe { raw::gas_used() }
}

pub fn get_call_result_size() -> u32 {
    unsafe { raw::call_result_size() }
}

pub struct HttpCallOptions {
    pub method: Option<String>,
    pub body: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct RawCallResult {
    status: i32,
    result: String,
    error: bool,
}

#[derive(Debug)]
pub struct CallResult {
    pub status: i32,
    pub result: String,
}

#[derive(Debug)]
pub struct CallResultError {
    pub error: String,
    pub status: i32,
}

pub fn http_call(url: &str, options: Option<HttpCallOptions>) -> Result<CallResult, CallResultError> {
    let unwrapped_options = options.unwrap_or(HttpCallOptions {
        method: Some("GET".to_string()),
        body: None,
        headers: None,
    });

    let call_data = json!({
        "url": url,
        "type": "http",
        "options": {
            "headers": unwrapped_options.headers,
            "body": unwrapped_options.body,
            "method": unwrapped_options.method.unwrap_or("GET".to_string()),
        }
    }).to_string();
    
    unsafe {
        raw::http_call(call_data.as_ptr(), call_data.len() as i32);
    }

    let mut result_data: Vec<u8> = Vec::new();
    result_data.resize(get_call_result_size() as usize, 0);

    unsafe {
        raw::call_result_copy(result_data.as_mut_ptr())
    }

    let result = str::from_utf8(&result_data);
    let unwrapped_result = result.unwrap_or("ERR_PARSING_RESULT");

    let parsed_result: RawCallResult = from_str(unwrapped_result).unwrap();

    if parsed_result.error {
        return Err(CallResultError {
            error: parsed_result.result,
            status: parsed_result.status,
        });
    }

    return Ok(CallResult {
        status: parsed_result.status,
        result: parsed_result.result,
    });
}

pub enum Outcome {
    Invalid,
    Valid(String)
}

pub fn exit_with_outcome(outcome: &Outcome) {
    let result = match outcome {
        Outcome::Invalid => json!({ "type": "Invalid" }).to_string(),
        Outcome::Valid(answer) => json!({ "type": "Valid", "value": answer }).to_string(),
    };

    let exit_code = match outcome {
        Outcome::Invalid => 1,
        Outcome::Valid(_) => 0,
    };
    
    // Make sure we don't interfene with other logs by adding \n
    print!("\n{}", result);
    process::exit(exit_code);
}
