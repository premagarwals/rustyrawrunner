use std::collections::HashMap;
use serde_json::{from_str, json, Value};

use crate::network::{Request, Response, VERSION};
use crate::models::codehandler::CodeHandler;
use crate::models::user::User;

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
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'username'".to_string(), String::from(VERSION)),
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'password'".to_string(), String::from(VERSION)),
    };

    let mut user = User::new(username, password);
    
    match user.register() {
        Ok(token) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            Response::new(
                200,
                headers,
                format!(r#"{{"token": "{}"}}"#, token),
                String::from(VERSION)
            )
        },
        Err(e) => {
            let status = if e.contains("already taken") { 409 } else { 500 };
            Response::new(status, HashMap::new(), e, String::from(VERSION))
        }
    }
}

pub fn login(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".to_string(), String::from(VERSION)),
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'username'".to_string(), String::from(VERSION)),
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'password'".to_string(), String::from(VERSION)),
    };

    let user = User::new(username, password);
    
    match user.login() {
        Ok(token) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            Response::new(
                200,
                headers,
                format!(r#"{{"token": "{}"}}"#, token),
                String::from(VERSION)
            )
        },
        Err(e) => {
            let status = if e.contains("Invalid username or password") { 401 } else { 500 };
            Response::new(status, HashMap::new(), e, String::from(VERSION))
        }
    }
}

pub fn ide(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".into(), VERSION.into()),
    };

    let code = match data.remove("code") {
        Some(Value::String(s)) => s,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'code'".into(), VERSION.into()),
    };

    let language = match data.remove("language") {
        Some(Value::String(s)) => s.to_lowercase(),
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'language'".into(), VERSION.into()),
    };

    let input = match data.remove("input") {
        Some(Value::String(i)) => i,
        _ => String::new(),
    };

    let mut handler = CodeHandler::new(code, language);
    
    handler.use_input(input);
    handler.execute();

    let response_body = json!({
        "output": handler.get_output(),
        "error": handler.get_error(),
        "runtime": handler.get_runtime(),
        "memory": handler.get_memory(),
    });

    let mut headers = HashMap::new();
    headers.insert("Content-Type".into(), "application/json".into());

    Response::new(200, headers, response_body.to_string(), VERSION.into())
}

