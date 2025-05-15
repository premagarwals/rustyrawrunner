use std::collections::HashMap;
use serde_json::{from_str, Value};
use serde::{Serialize};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use mysql::prelude::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use crate::network::{Request, Response, VERSION};

pub fn greet(request: &Request, _pool: &mysql::Pool) -> Response {
    Response::new(200, HashMap::new(), format!("Hello, world!\n\n<-- {}{} -->", request.get_header("Host").unwrap(), request.get_path()), String::from(VERSION))
}

pub fn not_found(request: &Request, _pool: &mysql::Pool) -> Response {
    Response::new(404, HashMap::new(), format!("Not found: {}", request.get_path()), String::from(VERSION))
}

pub fn signup(request: &Request, pool: &mysql::Pool) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".to_string(), String::from(VERSION)),
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'username'".to_string(), String::from(VERSION)),
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'password'".to_string(), String::from(VERSION)),
    };

    let hashed_password = match hash(&password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to hash password".to_string(), String::from(VERSION)),
    };

    let mut conn = match pool.get_conn() {
        Ok(c) => c,
        Err(_) => return Response::new(500, HashMap::new(), "DB connection failed".to_string(), String::from(VERSION)),
    };

    let exists: Option<u64> = match conn.exec_first(
        "SELECT COUNT(*) FROM users WHERE username = ?",
        (username.clone(),),
    ) {
        Ok(count) => count,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to check existing users".to_string(), String::from(VERSION)),
    };

    if let Some(count) = exists {
        if count > 0 {
            return Response::new(409, HashMap::new(), "Username already taken".to_string(), String::from(VERSION));
        }
    }

    if let Err(_) = conn.exec_drop(
        "INSERT INTO users (username, password) VALUES (?, ?)",
        (username.clone(), hashed_password),
    ) {
        return Response::new(500, HashMap::new(), "Failed to save user".to_string(), String::from(VERSION));
    }

    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 30))
        .expect("valid timestamp")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards 🙃")
        .as_secs() as usize;

    #[derive(Serialize)]
    struct Claims {
        username: String,
        exp: usize,
    }

    let claims = Claims {
        username: username.clone(),
        exp: expiration,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())) {
        Ok(t) => t,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to create JWT".to_string(), String::from(VERSION)),
    };

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response::new(
        200,
        headers,
        format!(r#"{{"token": "{}"}}"#, token),
        String::from(VERSION),
    )
}

pub fn login(request: &Request, pool: &mysql::Pool) -> Response {
    let mut data: HashMap<String, Value> = match from_str(request.get_body()) {
        Ok(json) => json,
        Err(_) => return Response::new(400, HashMap::new(), "Invalid JSON".to_string(), String::from(VERSION)),
    };

    let username = match data.remove("username") {
        Some(Value::String(u)) => u,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'username'".to_string(), String::from(VERSION)),
    };

    let password = match data.remove("password") {
        Some(Value::String(p)) => p,
        _ => return Response::new(400, HashMap::new(), "Missing or invalid 'password'".to_string(), String::from(VERSION)),
    };

    let mut conn = match pool.get_conn() {
        Ok(c) => c,
        Err(_) => return Response::new(500, HashMap::new(), "DB connection failed".to_string(), String::from(VERSION)),
    };

    let stored_hash: Option<String> = match conn.exec_first(
        "SELECT password FROM users WHERE username = ?",
        (username.clone(),),
    ) {
        Ok(hash_opt) => hash_opt,
        Err(_) => return Response::new(500, HashMap::new(), "DB query failed".to_string(), String::from(VERSION)),
    };

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Response::new(401, HashMap::new(), "Invalid username or password".to_string(), String::from(VERSION)),
    };

    if !bcrypt::verify(&password, &stored_hash).unwrap_or(false) {
        return Response::new(401, HashMap::new(), "Invalid username or password".to_string(), String::from(VERSION));
    }

    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 30))
        .expect("valid timestamp")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards 🙃")
        .as_secs() as usize;

    #[derive(Serialize)]
    struct Claims {
        username: String,
        exp: usize,
    }

    let claims = Claims {
        username: username.clone(),
        exp: expiration,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())) {
        Ok(t) => t,
        Err(_) => return Response::new(500, HashMap::new(), "Failed to create JWT".to_string(), String::from(VERSION)),
    };

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    Response::new(
        200,
        headers,
        format!(r#"{{"token": "{}"}}"#, token),
        String::from(VERSION),
    )
}

