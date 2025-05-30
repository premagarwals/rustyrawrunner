use crate::database::get_pool;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::env;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use mysql::prelude::Queryable;
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    username: String,
    exp: usize,
}

pub struct User {
    pub username: String,
    pub password: String,
    pub solves: Vec<u64>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            solves: Vec::new(),
        }
    }

    fn hash_password(&self) -> Result<String, String> {
        hash(&self.password, DEFAULT_COST)
            .map_err(|e| format!("Password hashing failed: {}", e))
    }

    fn create_jwt(&self) -> Result<String, String> {
        let expiration = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 30))
            .ok_or("Failed to calculate expiration")?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Time error")?
            .as_secs() as usize;

        let claims = Claims {
            username: self.username.clone(),
            exp: expiration,
        };

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET not set")?;

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes())
        ).map_err(|e| format!("JWT creation failed: {}", e))
    }

    pub fn register(&mut self) -> Result<String, String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        let exists: Option<u64> = conn
            .exec_first(
                "SELECT COUNT(*) FROM users WHERE username = ?",
                (&self.username,)
            )
            .map_err(|e| format!("Database query failed: {}", e))?;

        if exists.unwrap_or(0) > 0 {
            return Err("Username already taken".to_string());
        }

        let hashed_password = self.hash_password()?;

        conn.exec_drop(
            "INSERT INTO users (username, password) VALUES (?, ?)",
            (&self.username, &hashed_password)
        ).map_err(|e| format!("Failed to create user: {}", e))?;

        self.create_jwt()
    }

    pub fn login(&self) -> Result<String, String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        let result: Option<String> = conn
            .exec_first(
                "SELECT password FROM users WHERE username = ?",
                (&self.username,)
            )
            .map_err(|e| format!("Database query failed: {}", e))?;

        let hash = result.ok_or("Invalid username or password")?;

        if !verify(&self.password, &hash)
            .map_err(|e| format!("Password verification failed: {}", e))? {
            return Err("Invalid username or password".to_string());
        }

        self.create_jwt()
    }

    pub fn get_username_from_jwt(jwt: &str) -> Result<String, String> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET not set")?;

        let token_data = decode::<Claims>(
            jwt,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default()
        ).map_err(|e| format!("JWT decoding failed: {}", e))?;

        Ok(token_data.claims.username)
    }

    pub fn new_solve(&mut self, problem_id: u64) -> Result<(), String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        if !self.solves.contains(&problem_id) {
            self.solves.push(problem_id);
            
            conn.exec_drop(
                r"UPDATE users 
                  SET solves = JSON_ARRAY_APPEND(
                      IF(solves IS NULL, '[]', solves),
                      '$',
                      ?
                  )
                  WHERE username = ? AND NOT JSON_CONTAINS(solves, CAST(? AS JSON))",
                (problem_id, &self.username, problem_id)
            ).map_err(|e| format!("Failed to update solves: {}", e))?;
        }

        Ok(())
    }

    pub fn get_solves(&self) -> &Vec<u64> {
        &self.solves
    }

    pub fn get_user_by_username(username: &str) -> Result<User, String> {
        let mut conn = get_pool()
            .get_conn()
            .map_err(|e| format!("Database connection failed: {}", e))?;

        let (password, solves_json): (String, String) = conn
            .exec_first(
                "SELECT password, IFNULL(solves, '[]') FROM users WHERE username = ?",
                (username,)
            )
            .map_err(|e| format!("Database query failed: {}", e))?
            .ok_or("User not found")?;

        let solves: Vec<u64> = serde_json::from_str(&solves_json)
            .map_err(|e| format!("Failed to parse solves: {}", e))?;

        Ok(User {
            username: username.to_string(),
            password,
            solves,
        })
    }
}

