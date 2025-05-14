use crate::views;
use crate::network::{Request, Response, Method};

pub struct Router;

impl Router {
    pub fn new() -> Router {
        Router {}
    }
    pub fn route(request: &Request) -> Response {
        let handler = Self::path_to_handler(&request.get_path(), &request.get_method());
        handler(request)
    }

    fn path_to_handler(path: &str, method: &Method) -> fn(&Request) -> Response {
       match (path, method) {
           ("/", &Method::GET)  => views::greet,
           (_,_) => views::not_found,
       } 
    }

}
