use crate::config::FossilConfig;
use crate::error::{FossilP2pError, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct FossilCli {
    pub fossil_path: String,
}

impl FossilCli {
    pub fn new(config: &FossilConfig) -> Self {
        Self {
            fossil_path: config.fossil_path.clone(),
        }
    }

    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            fossil_path: path.into(),
        }
    }

    fn run_fossil(&self, args: &[&str], cwd: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new(&self.fossil_path);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output().map_err(|e| {
            FossilP2pError::Fossil(format!(
                "failed to execute '{}': {e}",
                self.fossil_path
            ))
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FossilP2pError::Fossil(format!(
                "fossil {} failed: {stderr}",
                args.first().unwrap_or(&"<no args>")
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn init(&self, path: &Path, name: &str) -> Result<()> {
        self.run_fossil(&["init", &format!("{}", path.display())], None)?;
        if !name.is_empty() {
            self.run_fossil(
                &["settings", "project-name", name],
                Some(path),
            )?;
        }
        Ok(())
    }

    pub fn open(&self, repo_path: &Path) -> Result<()> {
        if !repo_path.exists() {
            return Err(FossilP2pError::Fossil(format!(
                "repository not found: {}",
                repo_path.display()
            )));
        }
        Ok(())
    }

    pub fn status(&self, repo_path: &Path) -> Result<String> {
        self.run_fossil(&["status"], Some(repo_path))
    }

    pub fn add(&self, repo_path: &Path, paths: &[&str]) -> Result<()> {
        let mut args = vec!["add"];
        args.extend_from_slice(paths);
        self.run_fossil(&args, Some(repo_path))?;
        Ok(())
    }

    pub fn commit(&self, repo_path: &Path, message: &str, all: bool) -> Result<String> {
        let mut args = vec!["commit", "-m", message];
        if all {
            args.push("--all");
        }
        self.run_fossil(&args, Some(repo_path))
    }

    pub fn timeline(&self, repo_path: &Path, count: Option<usize>) -> Result<String> {
        match count {
            Some(n) => {
                let n_str = n.to_string();
                self.run_fossil(&["timeline", "-n", &n_str], Some(repo_path))
            }
            None => self.run_fossil(&["timeline"], Some(repo_path)),
        }
    }

    pub fn branches(&self, repo_path: &Path) -> Result<String> {
        self.run_fossil(&["branch", "list"], Some(repo_path))
    }

    pub fn clone(&self, url: &str, dest: &Path) -> Result<()> {
        self.run_fossil(
            &["clone", url, &format!("{}", dest.display())],
            None,
        )?;
        Ok(())
    }

    pub fn sync(&self, repo_path: &Path, transport_cmd: Option<&str>) -> Result<String> {
        if let Some(_cmd) = transport_cmd {
            self.run_fossil(&["sync", "--transport-command", _cmd], Some(repo_path))
        } else {
            self.run_fossil(&["sync"], Some(repo_path))
        }
    }

    pub fn serve(&self, repo_path: &Path, port: u16) -> Result<()> {
        self.run_fossil(
            &[
                "server",
                "--localhost",
                "-P",
                &port.to_string(),
            ],
            Some(repo_path),
        )?;
        Ok(())
    }

    pub fn info(&self, repo_path: &Path) -> Result<String> {
        self.run_fossil(&["info"], Some(repo_path))
    }

    pub fn ui(&self, repo_path: &Path, port: u16) -> Result<()> {
        self.run_fossil(
            &[
                "ui",
                "--localhost",
                "--port",
                &port.to_string(),
            ],
            Some(repo_path),
        )?;
        Ok(())
    }

    pub fn test_http(&self, repo_path: &Path, request_file: &Path) -> Result<String> {
        self.run_fossil(
            &["test-http", &format!("{}", request_file.display())],
            Some(repo_path),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fossil_cli_new() {
        let config = FossilConfig::default();
        let cli = FossilCli::new(&config);
        assert_eq!(cli.fossil_path, "fossil");
    }

    #[test]
    fn fossil_cli_custom_path() {
        let cli = FossilCli::with_path("/usr/local/bin/fossil");
        assert_eq!(cli.fossil_path, "/usr/local/bin/fossil");
    }
}
