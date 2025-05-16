use crate::database::get_pool;

pub struct Problem {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub input: String,
    pub output: String,
    pub solved: u64,
    pub tried: u64,
}

impl Problem {
    pub fn new(id: u64, title: String, description: String, input: String, output: String, solved: u64, tried: u64) -> Self {
        Self {
            id,
            title,
            description,
            input,
            output,
            solved,
            tried,
        }
    }
}
