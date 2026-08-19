use crate::error::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Home {
    pub path: PathBuf,
}

impl Home {
    pub fn new() -> Result<Self> {
        let path = if let Ok(home) = std::env::var("PEERGIT_HOME") {
            PathBuf::from(home)
        } else {
            let dirs = directories::ProjectDirs::from("", "", "peergit")
                .ok_or_else(|| crate::error::FossilP2pError::Config(
                    "cannot determine home directory".into(),
                ))?;
            dirs.data_dir().to_path_buf()
        };
        Ok(Self { path })
    }

    pub fn from_path(path: PathBuf) -> Result<Self> {
        Ok(Self { path })
    }

    pub fn storage(&self) -> PathBuf {
        self.path.join("storage")
    }

    pub fn config(&self) -> PathBuf {
        self.path.join("config.json")
    }

    pub fn keys(&self) -> PathBuf {
        self.path.join("keys")
    }

    pub fn db(&self) -> PathBuf {
        self.path.join("node.db")
    }

    pub fn secret_key_path(&self) -> PathBuf {
        self.keys().join("fossil-p2p")
    }

    pub fn public_key_path(&self) -> PathBuf {
        self.keys().join("fossil-p2p.pub")
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.storage())?;
        fs::create_dir_all(self.keys())?;
        fs::create_dir_all(self.path.join("repos"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn home_with_env_override() {
        let tmp = std::env::temp_dir().join("peergit-test-home");
        env::set_var("PEERGIT_HOME", &tmp);
        let home = Home::new().unwrap();
        assert_eq!(home.path, tmp);
        env::remove_var("PEERGIT_HOME");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn home_paths() {
        let tmp = std::env::temp_dir().join("peergit-test-paths");
        let home = Home::from_path(tmp.clone()).unwrap();
        assert!(home.config().to_string_lossy().contains("config.json"));
        assert!(home.db().to_string_lossy().contains("node.db"));
        assert!(home.keys().to_string_lossy().contains("keys"));
        let _ = fs::remove_dir_all(&tmp);
    }
}
