use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DockerError {
    #[error("Docker command failed: {0}")]
    CommandFailed(String),
    #[error("Failed to execute docker command: {0}")]
    ExecutionFailed(#[from] std::io::Error),
}

pub struct DockerCompose {
    project_dir: String,
    project_name: String,
}

impl DockerCompose {
    pub fn new(project_dir: &str, project_name: &str) -> Self {
        Self {
            project_dir: project_dir.to_string(),
            project_name: project_name.to_string(),
        }
    }

    pub fn up(&self) -> Result<(), DockerError> {
        // Execute the docker-compose command and capture the output
        let output = Command::new("docker-compose")
            .current_dir(&self.project_dir)
            .arg("-p")
            .arg(&self.project_name)
            .args(["up", "-d", "--wait"])
            .output()?; // This captures stdout and stderr

        // Check if the command was successful
        if !output.status.success() {
            // Convert stderr from bytes to a String
            let stderr = match std::str::from_utf8(&output.stderr) {
                Ok(s) => s.trim().to_string(),
                Err(_) => "Failed to parse error message".to_string(),
            };

            // Optionally, include stdout if needed
            let stdout = match std::str::from_utf8(&output.stdout) {
                Ok(s) => s.trim().to_string(),
                Err(_) => "Failed to parse output".to_string(),
            };

            // Construct a detailed error message
            let error_message = if !stderr.is_empty() {
                format!("{} | stdout: {}", stderr, stdout)
            } else {
                format!("Command failed without error message. stdout: {}", stdout)
            };

            return Err(DockerError::CommandFailed(error_message));
        }

        Ok(())
    }

    pub fn down(&self) -> Result<(), DockerError> {
        let status = Command::new("docker-compose")
            .current_dir(&self.project_dir)
            .arg("-p")
            .arg(&self.project_name)
            .arg("down")
            .status()?;

        if !status.success() {
            return Err(DockerError::CommandFailed(
                "docker-compose down failed".into(),
            ));
        }
        Ok(())
    }
}
