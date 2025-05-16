use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use uuid::Uuid;
use std::thread;
use std::sync::mpsc;

const TIMEOUT_SECS: u64 = 3;

pub enum Language {
    Cpp,
    Python,
    Java,
}

pub struct CodeHandler {
    code: String,
    lang: Language,
    input: String,
    output: String,
    error: String,
    runtime: String,
    memory: String,
}

impl CodeHandler {
    pub fn new(code: String, language: String) -> CodeHandler {
        let lang = match language.as_str() {
            "cpp" => Language::Cpp,
            "python" => Language::Python,
            "java" => Language::Java,
            _ => Language::Cpp,
        };
        CodeHandler {
            code: code.to_string(),
            lang,
            input: String::new(),
            output: String::new(),
            error: String::new(),
            runtime: String::new(),
            memory: String::new(),
        }
    }

    pub fn use_code(&mut self, code: String) {
        self.code = code;
    }
    
    pub fn use_input(&mut self, input: String) {
        self.input = input;
    }
    
    pub fn use_language(&mut self, language: Language) {
        self.lang = language;
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    pub fn get_error(&self) -> String {
        self.error.clone()
    }
    
    pub fn get_runtime(&self) -> String {
        self.runtime.clone()
    }

    pub fn get_memory(&self) -> String {
        self.memory.clone()
    }

    pub fn execute(&mut self) {
        match self.lang {
            Language::Cpp => {
                let id = Uuid::new_v4().to_string();
                let cpp_path = format!("/tmp/{}.cpp", id);
                let out_path = format!("/tmp/{}", id);

                if let Err(e) = fs::write(&cpp_path, &self.code) {
                    self.error = format!("Failed to write file: {}", e);
                    return;
                }

                let compile_output = Command::new("g++")
                    .arg(&cpp_path)
                    .arg("-o")
                    .arg(&out_path)
                    .stderr(Stdio::piped())
                    .output();

                match compile_output {
                    Ok(output) => {
                        if !output.status.success() {
                            self.error = String::from_utf8_lossy(&output.stderr).to_string();
                            return;
                        }
                    }
                    Err(e) => {
                        self.error = format!("Failed to compile: {}", e);
                        return;
                    }
                }

                let (tx, rx) = mpsc::channel();
                let start = Instant::now();

                let child = match Command::new(&out_path)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(mut c) => {
                        if let Some(stdin) = c.stdin.as_mut() {
                            if let Err(e) = stdin.write_all(self.input.as_bytes()) {
                                self.error = format!("Failed to write to stdin: {}", e);
                                return;
                            }
                        }
                        c
                    }
                    Err(e) => {
                        self.error = format!("Failed to start process: {}", e);
                        return;
                    }
                };

                // 👇 Wrap in Arc<Mutex<Option<Child>>>
                let child_arc = std::sync::Arc::new(std::sync::Mutex::new(Some(child)));
                let child_clone = std::sync::Arc::clone(&child_arc);
                let tx_clone = tx.clone();

                thread::spawn(move || {
                    let mut locked = child_clone.lock().unwrap();
                    if let Some(child_proc) = locked.take() {
                        let result = child_proc.wait_with_output();
                        let _ = tx_clone.send(result);
                    }
                });

                match rx.recv_timeout(Duration::from_secs(TIMEOUT_SECS)) {
                    Ok(Ok(output)) => {
                        self.output = String::from_utf8_lossy(&output.stdout).to_string();
                        self.error = String::from_utf8_lossy(&output.stderr).to_string();
                        self.runtime = format!("{}ms", start.elapsed().as_millis());
                        self.memory = "N/A".to_string();
                    }
                    Ok(Err(e)) => {
                        self.error = format!("Execution failed: {}", e);
                    }
                    Err(_) => {
                        let mut locked = child_arc.lock().unwrap();
                        if let Some(mut child_proc) = locked.take() {
                            let _ = child_proc.kill();
                        }
                        self.error = "TLE: Time Limit Exceeded".to_string();
                        self.runtime = format!(">{}s", TIMEOUT_SECS);
                        self.memory = "N/A".to_string();
                    }
                }

                let _ = fs::remove_file(&cpp_path);
                let _ = fs::remove_file(&out_path);
            }

            _ => {
                self.error = "Unsupported language".to_string();
            }
        }
    }

}
