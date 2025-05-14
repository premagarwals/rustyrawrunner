use serde_json::{Value, from_str};

use crate::network::{Request, Response, VERSION};
use std::collections::HashMap;

pub fn greet(request: &Request) -> Response {
    Response::new(200, HashMap::new(), format!("Hello, world!\n\n<-- {}{} -->", request.get_header("Host").unwrap(), request.get_path()), String::from(VERSION))
}

pub fn not_found(request: &Request) -> Response {
    Response::new(404, HashMap::new(), format!("Not found: {}", request.get_path()), String::from(VERSION))
}

pub fn signup(request: &Request) -> Response {
    let (username, password) = match from_str::<Value>(&request.get_body()) {
        Ok(json) => {
            let jsonuser = json.get("username").and_then(|v| v.as_str());
            let jsonpass = json.get("password").and_then(|v| v.as_str());

            match (jsonuser, jsonpass) {
                (Some(u), Some(p)) => (u.to_string(), p.to_string()),
                _ => {
                    return Response::new(
                        400,
                        HashMap::new(),
                        "Incomplete Data: username or password not provided".into(),
                        VERSION.into(),
                    );
                }
            }
        }
        Err(e) => {
            return Response::new(
                400,
                HashMap::new(),
                format!("Invalid JSON: {e}"),
                VERSION.into(),
            );
        }
    };

    Response::new(
        200,
        HashMap::new(),
        format!("{}, {}", username, password),
        VERSION.into(),
    )
}

