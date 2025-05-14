use serde_json::{from_str, Value};

use crate::network::{Request, Response, VERSION};
use std::collections::HashMap;

pub fn greet(request: &Request) -> Response {
    Response::new(200, HashMap::new(), format!("Hello, world!\n\n<-- {}{} -->", request.get_header("Host").unwrap(), request.get_path()), String::from(VERSION))
}

pub fn not_found(request: &Request) -> Response {
    Response::new(404, HashMap::new(), format!("Not found: {}", request.get_path()), String::from(VERSION))
}

pub fn signup(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".to_string(), String::from(VERSION)),
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => return Response::new(400, HashMap::new(), "Invalid JSON: Missing or invalid 'username'".to_string(), String::from(VERSION)),
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => return Response::new(400, HashMap::new(), "Invalid JSON: Missing or invalid 'password'".to_string(), String::from(VERSION)),
    };

    Response::new(
        200,
        HashMap::new(),
        format!("{}, {}", username, password),
        VERSION.into(),
    )
}

