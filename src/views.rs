use std::collections::HashMap;
use serde_json::{from_str, json, Value};

use crate::network::{Request, Response, VERSION};
use crate::models::codehandler::CodeHandler;
use crate::models::user::User;
use crate::models::problem::Problem;

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

pub fn add_problem(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".into(), VERSION.into()),
    };

    let token = match request.get_header("Authorization") {
        Some(t) => t.split_whitespace().nth(1).unwrap_or(""),
        None => return Response::new(401, HashMap::new(), "Missing or invalid 'Authorization' header".into(), VERSION.into()),
    };

    let creator = match User::get_username_from_jwt(&token) {
        Ok(username) => username,
        Err(_) => return Response::new(401, HashMap::new(), "Invalid token".into(), VERSION.into()),
    };

    let title = match data.remove("title") {
        Some(Value::String(s)) => s,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'title'".into(), VERSION.into()),
    };

    let description = match data.remove("description") {
        Some(Value::String(s)) => s,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'description'".into(), VERSION.into()),
    };

    let input = match data.remove("input") {
        Some(Value::String(s)) => s,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'input'".into(), VERSION.into()),
    };

    let output = match data.remove("output") {
        Some(Value::String(s)) => s,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'output'".into(), VERSION.into()),
    };

    let mut problem = Problem::new(creator, title, description, input, output);
    
    match problem.save() {
        Ok(_) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".into(), "application/json".into());
            let response_body = json!({
                "id": problem.id,
                "message": "Problem added successfully"
            });
            Response::new(201, headers, response_body.to_string(), VERSION.into())
        },
        Err(e) => Response::new(500, HashMap::new(), e, VERSION.into()),
    }
}

pub fn get_all_problems(_request: &Request) -> Response {
    match Problem::get_all() {
        Ok(problems) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".into(), "application/json".into());
            let response_body = json!({
                "problems": problems,
                "count": problems.len()
            });
            Response::new(200, headers, response_body.to_string(), VERSION.into())
        },
        Err(e) => Response::new(500, HashMap::new(), e, VERSION.into()),
    }
}

pub fn handle_options(_request: &Request) -> Response {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());
    Response::new(204, headers, String::new(), VERSION.to_string())
}

