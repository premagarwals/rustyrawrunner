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
    let response_body = json!({
        "message": format!("Not found: {}", request.get_path())
    });
    Response::new(404, HashMap::new(), response_body.to_string(), String::from(VERSION))
}

pub fn signup(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => {
            let response_body = json!({ "message": "Invalid JSON" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'username'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'password'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let mut user = User::new(username, password);
    
    match user.register() {
        Ok(token) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let response_body = json!({
                "message": "Registration successful",
                "token": token
            });
            Response::new(200, headers, response_body.to_string(), String::from(VERSION))
        },
        Err(e) => {
            let status = if e.contains("already taken") { 409 } else { 500 };
            let response_body = json!({ "message": e });
            Response::new(status, HashMap::new(), response_body.to_string(), String::from(VERSION))
        }
    }
}

pub fn login(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => {
            let response_body = json!({ "message": "Invalid JSON" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'username'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'password'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), String::from(VERSION));
        }
    };

    let user = User::new(username, password);
    
    match user.login() {
        Ok(token) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            let response_body = json!({
                "message": "Login successful",
                "token": token
            });
            Response::new(200, headers, response_body.to_string(), String::from(VERSION))
        },
        Err(e) => {
            let status = if e.contains("Invalid username or password") { 401 } else { 500 };
            let response_body = json!({ "message": e });
            Response::new(status, HashMap::new(), response_body.to_string(), String::from(VERSION))
        }
    }
}

pub fn ide(request: &Request) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => {
            let response_body = json!({ "message": "Invalid JSON" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let code = match data.remove("code") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'code'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let language = match data.remove("language") {
        Some(Value::String(s)) => s.to_lowercase(),
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'language'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let input = match data.remove("input") {
        Some(Value::String(i)) => i,
        _ => String::new(),
    };

    let mut handler = CodeHandler::new(code, language);
    
    handler.use_input(input);
    handler.execute();

    let response_body = json!({
        "message": if handler.get_error().is_empty() { "Execution successful" } else { "Execution failed" },
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
        Err(_) => {
            let response_body = json!({ "message": "Invalid JSON" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let token = match request.get_header("Authorization") {
        Some(t) => t.split_whitespace().nth(1).unwrap_or(""),
        None => {
            let response_body = json!({ "message": "Missing or invalid 'Authorization' header" });
            return Response::new(401, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let creator = match User::get_username_from_jwt(&token) {
        Ok(username) => username,
        Err(_) => {
            let response_body = json!({ "message": "Invalid token" });
            return Response::new(401, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let title = match data.remove("title") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'title'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let description = match data.remove("description") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'description'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let input = match data.remove("input") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'input'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let output = match data.remove("output") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'output'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let mut problem = Problem::new(creator, title, description, input, output);
    
    match problem.save() {
        Ok(_) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".into(), "application/json".into());
            let response_body = json!({
                "message": "Problem added successfully",
                "id": problem.id
            });
            Response::new(201, headers, response_body.to_string(), VERSION.into())
        },
        Err(e) => {
            let response_body = json!({ "message": e });
            Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into())
        }
    }
}

pub fn get_all_problems(_request: &Request) -> Response {
    match Problem::get_all() {
        Ok(problems) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".into(), "application/json".into());
            let response_body = json!({
                "message": "Problems retrieved successfully",
                "problems": problems,
                "count": problems.len()
            });
            Response::new(200, headers, response_body.to_string(), VERSION.into())
        },
        Err(e) => {
            let response_body = json!({ "message": e });
            Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into())
        }
    }
}

pub fn get_problem_by_id(request: &Request, id: u64) -> Response {
    match Problem::find_by_id(id) {
        Ok(Some(problem)) => {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".into(), "application/json".into());
            let response_body = json!({
                "message": "Problem retrieved successfully",
                "problem": problem
            });
            Response::new(200, headers, response_body.to_string(), VERSION.into())
        },
        Ok(None) => {
            let response_body = json!({
                "message": format!("Problem with id {} not found", id)
            });
            Response::new(404, HashMap::new(), response_body.to_string(), VERSION.into())
        },
        Err(e) => {
            let response_body = json!({ "message": e });
            Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into())
        }
    }
}

pub fn solve_problem(request: &Request, problem_id: u64) -> Response {
    let token = match request.get_header("Authorization") {
        Some(t) => t.split_whitespace().nth(1).unwrap_or(""),
        None => {
            let response_body = json!({ "message": "Missing or invalid 'Authorization' header" });
            return Response::new(401, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let username = match User::get_username_from_jwt(token) {
        Ok(u) => u,
        Err(e) => {
            let response_body = json!({ "message": e });
            return Response::new(401, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => {
            let response_body = json!({ "message": "Invalid JSON" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let code = match data.remove("code") {
        Some(Value::String(s)) => s,
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'code'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let language = match data.remove("language") {
        Some(Value::String(s)) => s.to_lowercase(),
        _ => {
            let response_body = json!({ "message": "Missing or invalid 'language'" });
            return Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    let problem = match Problem::find_by_id(problem_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            let response_body = json!({ "message": format!("Problem {} not found", problem_id) });
            return Response::new(404, HashMap::new(), response_body.to_string(), VERSION.into());
        }
        Err(e) => {
            let response_body = json!({ "message": e });
            return Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into());
        }
    };

    if let Err(e) = Problem::increment_tried(problem_id) {
        let response_body = json!({ "message": e });
        return Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into());
    }

    let mut handler = CodeHandler::new(code, language);
    handler.use_input(problem.input.clone());
    handler.execute();

    if handler.get_output().trim() == problem.output.trim() {
        let mut user = User::new(username, String::new());
        if let Err(e) = user.new_solve(problem_id) {
            let response_body = json!({ "message": e });
            return Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into());
        }

        if let Err(e) = Problem::increment_solved(problem_id) {
            let response_body = json!({ "message": e });
            return Response::new(500, HashMap::new(), response_body.to_string(), VERSION.into());
        }

        let mut headers = HashMap::new();
        headers.insert("Content-Type".into(), "application/json".into());
        let response_body = json!({
            "message": "Problem solved successfully!",
            "output": handler.get_output(),
            "runtime": handler.get_runtime(),
            "memory": handler.get_memory()
        });
        Response::new(200, headers, response_body.to_string(), VERSION.into())
    } else {
        let response_body = json!({
            "message": "Wrong answer",
            "output": handler.get_output(),
            "error": handler.get_error(),
            "runtime": handler.get_runtime(),
            "memory": handler.get_memory()
        });
        Response::new(400, HashMap::new(), response_body.to_string(), VERSION.into())
    }
}

pub fn handle_options(_request: &Request) -> Response {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());
    let response_body = json!({ "message": "Options request successful" });
    Response::new(204, headers, response_body.to_string(), VERSION.to_string())
}

