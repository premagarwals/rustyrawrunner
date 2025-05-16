use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIME_LIMIT: u64 = 2;

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

    pub fn execute(&mut self) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        
        match self.lang {
            Language::Cpp => {
                let temp_dir = std::env::temp_dir();
                let source_path = temp_dir.join("program.cpp");
                let input_path = temp_dir.join("input.txt");
                
                if let Err(e) = std::fs::write(&source_path, &self.code) {
                    self.error = format!("Failed to write source code: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                if let Err(e) = std::fs::write(&input_path, &self.input) {
                    self.error = format!("Failed to write input: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                let container_check = std::process::Command::new("docker")
                    .args(&["ps", "--filter", "name=code-sandbox", "--format", "{{.Names}}"])
                    .output();
                
                match container_check {
                    Ok(output) => {
                        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
                        if !output_str.contains("code-sandbox") {
                            self.error = "Sandbox container is not running. Please start it with docker-compose up -d".to_string();
                            self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                            return Err(self.error.clone());
                        }
                    },
                    Err(e) => {
                        self.error = format!("Failed to check if sandbox container is running: {}", e);
                        self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                        return Err(self.error.clone());
                    }
                }
                
                let copy_source = std::process::Command::new("docker")
                    .args(&[
                        "cp", 
                        source_path.to_str().unwrap(),
                        "code-sandbox:/sandbox/program.cpp"
                    ])
                    .output();
                
                if let Err(e) = copy_source {
                    self.error = format!("Failed to copy source to container: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                let copy_input = std::process::Command::new("docker")
                    .args(&[
                        "cp", 
                        input_path.to_str().unwrap(),
                        "code-sandbox:/sandbox/input.txt"
                    ])
                    .output();
                
                if let Err(e) = copy_input {
                    self.error = format!("Failed to copy input to container: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                let install_output = std::process::Command::new("docker")
                    .args(&[
                        "exec",
                        "code-sandbox",
                        "/bin/sh", "-c",
                        "which g++ || apk add --no-cache g++"
                    ])
                    .output();
                
                if let Err(e) = install_output {
                    self.error = format!("Failed to install g++: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                let compile_output = std::process::Command::new("docker")
                    .args(&[
                        "exec",
                        "code-sandbox",
                        "/bin/sh", "-c",
                        "cd /sandbox && g++ program.cpp -o program -std=c++17 2>&1"
                    ])
                    .output();
                
                match compile_output {
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let compile_output = if stderr.is_empty() { stdout } else { stderr };
                        
                        if !output.status.success() || compile_output.contains("error:") {
                            self.error = format!("Compilation error: {}", compile_output);
                            self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                            return Err(self.error.clone());
                        }
                    },
                    Err(e) => {
                        self.error = format!("Compilation failed: {}", e);
                        self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                        return Err(self.error.clone());
                    }
                }
                
                let chmod_output = std::process::Command::new("docker")
                    .args(&[
                        "exec",
                        "code-sandbox",
                        "/bin/sh", "-c",
                        "chmod +x /sandbox/program"
                    ])
                    .output();
                
                if let Err(e) = chmod_output {
                    self.error = format!("Failed to make binary executable: {}", e);
                    self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                    return Err(self.error.clone());
                }
                
                let run_cmd = format!(
                    "cd /sandbox && timeout -s KILL {} ./program < input.txt; exit_code=$?; \
                     if [ $exit_code -eq 124 ] || [ $exit_code -eq 137 ]; then \
                       echo 'Time limit exceeded' >&2; \
                       exit $exit_code; \
                     elif [ $exit_code -ne 0 ]; then \
                       echo 'Program exited with code '$exit_code >&2; \
                       exit $exit_code; \
                     fi", 
                    TIME_LIMIT
                );
                
                let run_output = std::process::Command::new("docker")
                    .args(&[
                        "exec",
                        "code-sandbox",
                        "/bin/sh", "-c", 
                        &run_cmd
                    ])
                    .output();
                
                self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                
                match run_output {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        
                        self.output = stdout;
                        
                        if stderr.contains("Time limit exceeded") {
                            self.error = "Time limit exceeded".to_string();
                        } else if !stderr.is_empty() {
                            self.error = stderr;
                        } else if !output.status.success() {
                            self.error = format!("Program exited with non-zero status: {}", output.status);
                        }
                        
                        let _ = std::fs::remove_file(&source_path);
                        let _ = std::fs::remove_file(&input_path);
                        
                        let _ = std::process::Command::new("docker")
                            .args(&[
                                "exec",
                                "code-sandbox",
                                "/bin/sh", "-c",
                                "rm -f /sandbox/program.cpp /sandbox/program /sandbox/input.txt"
                            ])
                            .output();
                        
                        Ok(())
                    },
                    Err(e) => {
                        self.error = format!("Failed to execute: {}", e);
                        Err(self.error.clone())
                    }
                }
            },
            _ => {
                self.error = "Invalid language provided. Only C++ is supported currently.".to_string();
                self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
                Err(self.error.clone())
            }
        }
    }
}

