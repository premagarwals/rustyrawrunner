use libc::{c_void, syscall};
use std::mem;
use mysql::*;
use mysql::prelude::*;
use dotenvy::dotenv;
use std::env;

mod network;
mod views;
mod routes;
mod models;

use network::Request;
use routes::Router;

const SYS_SOCKET: i64 = 41;
const SYS_BIND: i64 = 49;
const SYS_LISTEN: i64 = 50;
const SYS_ACCEPT: i64 = 43;
const SYS_READ: i64 = 0;
const SYS_WRITE: i64 = 1;
const SYS_CLOSE: i64 = 3;

const AF_INET: i32 = 2;
const SOCK_STREAM: i32 = 1;
const INADDR_ANY: u32 = 0;

#[repr(C)]
struct SockAddrIn {
    sin_family: u16,
    sin_port: u16,
    sin_addr: u32,
    sin_zero: [u8; 8],
}

fn htons(port: u16) -> u16 {
    port.to_be()
}

fn main() {
    dotenv().ok();

    let user = env::var("MYSQL_USER").expect("MYSQL_USER not set");
    let pass = env::var("MYSQL_PASSWORD").expect("MYSQL_PASSWORD not set");
    let host = env::var("MYSQL_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
    let db = env::var("MYSQL_DATABASE").expect("MYSQL_DATABASE not set");

    let url = format!("mysql://{user}:{pass}@{host}:{port}/{db}");

    let pool = Pool::new(url.as_str()).expect("Couldn't connect to DB");
    let mut conn = pool.get_conn().expect("No conn :(");

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(100),
            password TEXT
        )"
    ).unwrap();

    println!("Table connected, let’s gooo 😎");
    
    unsafe {
        let sockfd = syscall(SYS_SOCKET, AF_INET, SOCK_STREAM, 0) as i32;
        if sockfd < 0 {
            panic!("socket syscall failed");
        }

        let addr = SockAddrIn {
            sin_family: AF_INET as u16,
            sin_port: htons(8080),
            sin_addr: u32::from_be(INADDR_ANY),
            sin_zero: [0; 8],
        };

        let res = syscall(
            SYS_BIND,
            sockfd,
            &addr as *const _,
            mem::size_of::<SockAddrIn>() as u32,
        );
        if res < 0 {
            panic!("bind syscall failed");
        }

        let res = syscall(SYS_LISTEN, sockfd, 10);
        if res < 0 {
            panic!("listen syscall failed");
        }

        println!("Server listening on port 8080...");

        loop {
            let client_fd = syscall(SYS_ACCEPT, sockfd, 0 as *mut c_void, 0 as *mut c_void) as i32;
            if client_fd < 0 {
                eprintln!("accept failed");
                continue;
            }

            let mut buffer = [0u8; 1024];
            syscall(SYS_READ, client_fd, buffer.as_mut_ptr(), 1024);
            let request = Request::parse(std::str::from_utf8(&buffer).unwrap()).unwrap();
            println!("{:?}", request);
            let response = Router::route(&request, &pool).to_string();
            syscall(SYS_WRITE, client_fd, response.as_ptr(), response.len());

            syscall(SYS_CLOSE, client_fd);
        }
    }
}


