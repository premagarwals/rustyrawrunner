use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}


#[derive(Debug)]
pub struct Response {
    status: u16,
    version: String,
    headers: HashMap<String, String>,
    body: String,
}

pub const VERSION: &str = "HTTP/1.1";

impl Request {
    pub fn new(method: String, path: String, version: String, headers: HashMap<String, String>, body: String) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            body,
        }
    }

    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        let mut lines = raw.split("\r\n");

        let request_line = lines.next().ok_or("Missing request line")?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().ok_or("Missing method")?.to_string();
        let path = request_parts.next().ok_or("Missing path")?.to_string();
        let version = request_parts.next().ok_or("Missing version")?.to_string();

        let mut headers = HashMap::new();
        for line in &mut lines {
            if line.is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().ok_or("Malformed header")?.to_string();
            let value = parts.next().ok_or("Malformed header")?.to_string();
            headers.insert(key, value);
        }

        let body = lines.collect::<Vec<_>>().join("\r\n");

        Ok(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn get_body(&self) -> &String {
        &self.body
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{} {} {}\r\n", self.method, self.path, self.version);
        for (key, value) in &self.headers {
            result.push_str(&format!("{}: {}\r\n", key, value));
        }
        result.push_str("\r\n");
        result.push_str(&self.body);
        result
    }
}

impl Response {
    pub fn new(status: u16, headers: HashMap<String, String>, body: String, version: String) -> Self {
        Self {
            status,
            version,
            headers,
            body,
        }
    }

    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        let mut lines = raw.split("\r\n");

        let status_line = lines.next().ok_or("Missing status line")?;
        let mut status_parts = status_line.split_whitespace();
        let version = status_parts.next().ok_or("Missing version")?.to_string();
        let status = status_parts.next().ok_or("Missing status")?.parse::<u16>().unwrap();

        let mut headers = HashMap::new();
        for line in &mut lines {
            if line.is_empty() {
                break;
            }
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().ok_or("Malformed header")?.to_string();
            let value = parts.next().ok_or("Malformed header")?.to_string();
            headers.insert(key, value);
        }

        let body = lines.collect::<Vec<_>>().join("\r\n");

        Ok(Self {
            status,
            version,
            headers,
            body,
        })
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{} {} {}\r\n", self.version, self.status, Self::status_reason(self.status));
        for (key, value) in &self.headers {
            result.push_str(&format!("{}: {}\r\n", key, value));
        }
        result.push_str("\r\n");
        result.push_str(&self.body);
        result
    }


    fn status_reason(status: u16) -> &'static str {
        match status {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}
