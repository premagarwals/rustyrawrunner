use crate::views;
use crate::network::{Request, Response};

pub struct Router;

impl Router {
    pub fn new() -> Router {
        Router {}
    }
    pub fn route(request: &Request) -> Response {
        let handler = Self::path_to_handler(&request.get_path());
        handler(request)
    }

    fn path_to_handler(path: &str) -> fn(&Request) -> Response {
       match path {
            "/" => views::greet,
            _ => views::not_found,
       } 
    }

}
