use std::env;
use std::sync::OnceLock;
use mysql::*;
use mysql::prelude::*;

static POOL: OnceLock<Pool> = OnceLock::new();

pub fn init_db() {
    let user = env::var("MYSQL_USER").expect("MYSQL_USER not set");
    let pass = env::var("MYSQL_PASSWORD").expect("MYSQL_PASSWORD not set");
    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    let db = env::var("MYSQL_DATABASE").expect("MYSQL_DATABASE not set");

    let url = format!("mysql://{user}:{pass}@{host}:{port}/{db}");
    let pool = Pool::new(url.as_str()).expect("Couldn't connect to DB");

    POOL.set(pool).ok().expect("DB Pool already initialized");

    let mut conn = get_pool().get_conn().expect("No conn :(");
        
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS users (
            username VARCHAR(100) NOT NULL PRIMARY KEY,
            password TEXT NOT NULL,
            solves JSON DEFAULT ('[]')
        )"
    ).unwrap();

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS problems (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            creator VARCHAR(100) NOT NULL,
            title VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            input TEXT NOT NULL,
            output TEXT NOT NULL,
            solved BIGINT UNSIGNED DEFAULT 0,
            tried BIGINT UNSIGNED DEFAULT 0,
            INDEX creator_idx (creator),
            FOREIGN KEY (creator) REFERENCES users(username) ON DELETE CASCADE
        )"
    ).unwrap();

    println!("DB initialized");
}

pub fn get_pool() -> &'static Pool {
    POOL.get().expect("DB not initialized. Call init_db() first.")
}

