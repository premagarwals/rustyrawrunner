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
           ("/" | "", &Method::GET)  => views::greet,
           ("/signup/" | "/signup", &Method::POST) => views::signup,
           ("/login/" | "/login", &Method::POST) => views::login,
           ("/ide/" | "/ide", &Method::POST) => views::ide,
           ("/addproblem/" | "/addproblem", &Method::POST) => views::add_problem,
           ("/getproblems/" | "/getproblems", &Method::GET) => views::get_all_problems,
            (_, &Method::OPTIONS) => views::handle_options,
           (_,_) => views::not_found,
       } 
    }
}
