use crate::network::{Request, Response, VERSION};
use std::collections::HashMap;

pub fn greet(request: &Request) -> Response {
    Response::new(200, HashMap::new(), format!("Hello, world!\n\n<-- {}{} -->", request.get_header("Host").unwrap(), request.get_path()), String::from(VERSION))
}

pub fn not_found(request: &Request) -> Response {
    Response::new(404, HashMap::new(), format!("Not found: {}", request.get_path()), String::from(VERSION))
}
