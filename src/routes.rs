use crate::views;
use crate::network::{Request, Response, Method};

pub struct Router;

impl Router {
    pub fn new() -> Router {
        Router {}
    }
    pub fn route(request: &Request, pool: &mysql::Pool) -> Response {
        let handler = Self::path_to_handler(&request.get_path(), &request.get_method());
        handler(request, pool)
    }

    fn path_to_handler(path: &str, method: &Method) -> fn(&Request, &mysql::Pool) -> Response {
       match (path, method) {
           ("/", &Method::GET)  => views::greet,
           ("/signup", &Method::POST) => views::signup,
           ("/login", &Method::POST) => views::login,
           ("/ide", &Method::POST) => views::ide,
           (_,_) => views::not_found,
       } 
    }

}
