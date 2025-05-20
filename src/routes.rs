use crate::views;
use crate::network::{Request, Response, Method};

pub struct Router;

impl Router {
    pub fn new() -> Router {
        Router {}
    }
    pub fn route(request: &Request) -> Response {
        if let Some(problem_id) = Self::extract_problem_id(request.get_path()) {
            return match request.get_method() {
                Method::GET => views::get_problem_by_id(&request, problem_id),
                _ => views::not_found(&request),
            };
        }
        let handler = Self::path_to_handler(&request.get_path(), &request.get_method());
        handler(request)
    }

    fn path_to_handler(path: &str, method: &Method) -> fn(&Request) -> Response {
        match (path, method) {
            ("/" | "", &Method::GET) => views::greet,
            ("/signup/" | "/signup", &Method::POST) => views::signup,
            ("/login/" | "/login", &Method::POST) => views::login,
            ("/ide/" | "/ide", &Method::POST) => views::ide,
            ("/addproblem/" | "/addproblem", &Method::POST) => views::add_problem,
            ("/getproblems/" | "/getproblems", &Method::GET) => views::get_all_problems,
            (_, &Method::OPTIONS) => views::handle_options,
            (_, _) => views::not_found,
        }
    }

    fn extract_problem_id(path: &str) -> Option<u64> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 3 && parts[1] == "problem" {
            parts[2].parse().ok()
        } else {
            None
        }
    }
}
