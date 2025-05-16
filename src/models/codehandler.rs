use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::path::{PathBuf, Path};

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
        let result = match self.lang {
            Language::Cpp => self.execute_cpp(),
            Language::Python => self.execute_python(),
            Language::Java => self.execute_java(),
        };
        self.runtime = format!("{:.3}s", start_time.elapsed().as_secs_f64());
        result
    }

    fn prepare_files(&self) -> Result<(PathBuf, PathBuf), String> {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join(match self.lang {
            Language::Cpp => "program.cpp",
            Language::Python => "program.py",
            Language::Java => "Main.java",
        });
        let input_path = temp_dir.join("input.txt");

        std::fs::write(&source_path, &self.code)
            .map_err(|e| format!("Failed to write source code: {}", e))?;
        std::fs::write(&input_path, &self.input)
            .map_err(|e| format!("Failed to write input: {}", e))?;

        Ok((source_path, input_path))
    }

    fn execute_cpp(&mut self) -> Result<(), String> {
        let (source_path, input_path) = self.prepare_files()?;
        
        if let Err(e) = self.copy_to_container(&source_path, "/sandbox/program.cpp") {
            self.error = e;
            return Err(self.error.clone());
        }
        
        if let Err(e) = self.copy_to_container(&input_path, "/sandbox/input.txt") {
            self.error = e;
            return Err(self.error.clone());
        }

        if let Err(e) = self.run_in_container("g++ /sandbox/program.cpp -o /sandbox/program -std=c++17") {
            return Err(e);
        }

        self.run_and_capture(
            format!("cd /sandbox && timeout -s KILL {} ./program < input.txt", TIME_LIMIT),
            &source_path,
            &input_path,
        )
    }

    fn execute_python(&mut self) -> Result<(), String> {
        let (source_path, input_path) = self.prepare_files()?;
        
        if let Err(e) = self.copy_to_container(&source_path, "/sandbox/program.py") {
            self.error = e;
            return Err(self.error.clone());
        }
        
        if let Err(e) = self.copy_to_container(&input_path, "/sandbox/input.txt") {
            self.error = e;
            return Err(self.error.clone());
        }

        self.run_and_capture(
            format!("cd /sandbox && timeout -s KILL {} python3 program.py < input.txt", TIME_LIMIT),
            &source_path,
            &input_path,
        )
    }

    fn execute_java(&mut self) -> Result<(), String> {
        let (source_path, input_path) = self.prepare_files()?;
        
        if let Err(e) = self.copy_to_container(&source_path, "/sandbox/Main.java") {
            self.error = e;
            return Err(self.error.clone());
        }
        
        if let Err(e) = self.copy_to_container(&input_path, "/sandbox/input.txt") {
            self.error = e;
            return Err(self.error.clone());
        }

        if let Err(e) = self.run_in_container("cd /sandbox && javac Main.java") {
            return Err(e);
        }

        self.run_and_capture(
            format!("cd /sandbox && timeout -s KILL {} java Main < input.txt", TIME_LIMIT),
            &source_path,
            &input_path,
        )
    }

    fn copy_to_container(&self, host_path: &Path, container_path: &str) -> Result<(), String> {
        std::process::Command::new("docker")
            .args(&["cp", host_path.to_str().unwrap(), &format!("code-sandbox:{}", container_path)])
            .output()
            .map_err(|e| format!("Failed to copy file to container: {}", e))?;
        Ok(())
    }

    fn run_in_container(&mut self, cmd: &str) -> Result<(), String> {
        let status = std::process::Command::new("docker")
            .args(&["exec", "code-sandbox", "/bin/sh", "-c", cmd])
            .output()
            .map_err(|e| format!("Execution failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&status.stdout).to_string();
        let stderr = String::from_utf8_lossy(&status.stderr).to_string();
        
        if !status.status.success() {
            self.error = if stderr.is_empty() { stdout } else { stderr };
            return Err(self.error.clone());
        }

        Ok(())
    }

    fn run_and_capture(&mut self, cmd: String, src: &Path, inp: &Path) -> Result<(), String> {
        let output = std::process::Command::new("docker")
            .args(&["exec", "code-sandbox", "/bin/sh", "-c", &cmd])
            .output()
            .map_err(|e| {
                self.error = format!("Failed to run command: {}", e);
                self.error.clone()
            })?;

        self.output = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(0);

        // Cleanup first
        let _ = std::fs::remove_file(src);
        let _ = std::fs::remove_file(inp);
        let _ = std::process::Command::new("docker")
            .args(&["exec", "code-sandbox", "/bin/sh", "-c", "rm -f /sandbox/*"])
            .output();

        if exit_code == 124 || exit_code == 137 {
            self.error = "Time Limit Exceeded".to_string();
            return Err(self.error.clone());
        }

        if !stderr.is_empty() {
            self.error = stderr;
            return Err(self.error.clone());
        }

        if !output.status.success() {
            self.error = "Runtime Error".to_string();
            return Err(self.error.clone());
        }

        Ok(())
    }
}
