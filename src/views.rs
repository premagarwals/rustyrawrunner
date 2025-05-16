use std::collections::HashMap;
use serde_json::{from_str, json, Value};
use mysql::prelude::*;
use std::env;

use crate::network::{Request, Response, VERSION};
use crate::models::codehandler::CodeHandler;
use crate::database::get_pool;
use crate::utils::auth::{create_jwt_for, verify_password, hash_password};

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

    let hashed_password = match hash_password(&password) {
        Ok(h) => h,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to hash password".to_string(), String::from(VERSION)),
    };

    let mut conn = match get_pool().get_conn() {
        Ok(c) => c,
        Err(_) => return Response::new(500, HashMap::new(), "DB connection failed".to_string(), String::from(VERSION)),
    };

    let exists: Option<u64> = match conn.exec_first(
        "SELECT COUNT(*) FROM users WHERE username = ?",
        (username.clone(),),
    ) {
        Ok(count) => count,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to check existing users".to_string(), String::from(VERSION)),
    };

    if let Some(count) = exists {
        if count > 0 {
            return Response::new(409, HashMap::new(), "Username already taken".to_string(), String::from(VERSION));
        }
    }

    if let Err(_) = conn.exec_drop(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        (username.clone(), hashed_password),
    ) {
        return Response::new(500, HashMap::new(), "Failed to save user".to_string(), String::from(VERSION));
    }

    let token = match create_jwt_for(&username) {
        Ok(t) => t,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to create JWT".to_string(), String::from(VERSION)),
    };

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response::new(
        200,
        headers,
        format!(r#"{{"token": "{}"}}"#, token),
        String::from(VERSION),
    )
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

    let mut conn = match get_pool().get_conn() {
        Ok(c) => c,
        Err(_) => return Response::new(500, HashMap::new(), "DB connection failed".to_string(), String::from(VERSION)),
    };

    let stored_hash: Option<String> = match conn.exec_first(
        "SELECT password FROM users WHERE username = ?",
        (username.clone(),),
    ) {
        Ok(hash_opt) => hash_opt,
        Err(_) => return Response::new(500, HashMap::new(), "DB query failed".to_string(), String::from(VERSION)),
    };

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Response::new(401, HashMap::new(), "Invalid username or password".to_string(), String::from(VERSION)),
    };

    if !verify_password(&password, &stored_hash).unwrap_or(false) {
        return Response::new(401, HashMap::new(), "Invalid username or password".to_string(), String::from(VERSION));
    }

    let token = match create_jwt_for(&username) {
        Ok(t) => t,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to create JWT".to_string(), String::from(VERSION)),
    };

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response::new(
        200,
        headers,
        format!(r#"{{"token": "{}"}}"#, token),
        String::from(VERSION),
    )
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

