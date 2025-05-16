use crate::database::get_pool;
use mysql::prelude::*;
use mysql::{Row, FromRowError};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub id: Option<u64>,
    pub creator: String,
    pub title: String,
    pub description: String,
    pub input: String,
    pub output: String,
    pub solved: u64,
    pub tried: u64,
}

impl FromRow for Problem {
    fn from_row(row: Row) -> Self {
        Self::from_row_opt(row)
            .expect("Failed to convert database row to Problem")
    }

    fn from_row_opt(row: Row) -> Result<Self, FromRowError> {
        Ok(Problem {
            id: row.get("id"),
            creator: row.get("creator").ok_or(FromRowError(row.clone()))?,
            title: row.get("title").ok_or(FromRowError(row.clone()))?,
            description: row.get("description").ok_or(FromRowError(row.clone()))?,
            input: row.get("input").ok_or(FromRowError(row.clone()))?,
            output: row.get("output").ok_or(FromRowError(row.clone()))?,
            solved: row.get("solved").ok_or(FromRowError(row.clone()))?,
            tried: row.get("tried").ok_or(FromRowError(row.clone()))?,
        })
    }
}

impl Problem {
    pub fn new(creator: String, title: String, description: String, input: String, output: String) -> Self {
        Self {
            id: None,
            creator,
            title,
            description,
            input,
            output,
            solved: 0,
            tried: 0,
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        match self.id {
            // Update existing problem
            Some(id) => {
                conn.exec_drop(
                    "UPDATE problems SET title=?, description=?, input=?, output=?, solved=?, tried=? WHERE id=?",
                    (
                        &self.title,
                        &self.description,
                        &self.input,
                        &self.output,
                        &self.solved,
                        &self.tried,
                        id
                    ),
                ).map_err(|e| format!("Failed to update problem: {}", e))?;
            },
            // Insert new problem
            None => {
                conn.exec_drop(
                    "INSERT INTO problems (creator, title, description, input, output, solved, tried) 
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    (
                        &self.creator,
                        &self.title,
                        &self.description,
                        &self.input,
                        &self.output,
                        &self.solved,
                        &self.tried,
                    ),
                ).map_err(|e| format!("Failed to create problem: {}", e))?;

                self.id = Some(conn.last_insert_id());
            }
        }

        Ok(())
    }

    pub fn find_by_id(id: u64) -> Result<Option<Problem>, String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        conn.exec_first(
            "SELECT id, creator, title, description, input, output, solved, tried 
             FROM problems WHERE id = ?",
            (id,)
        ).map_err(|e| format!("Database query failed: {}", e))
    }

    pub fn get_all() -> Result<Vec<Problem>, String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        conn.exec(
            "SELECT id, creator, title, description, input, output, solved, tried 
             FROM problems ORDER BY id DESC",
            (),
        ).map_err(|e| format!("Database query failed: {}", e))
    }
}
