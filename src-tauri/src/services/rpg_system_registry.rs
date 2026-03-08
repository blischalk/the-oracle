use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;

use crate::domain::rpg_system::RpgSystem;

pub struct RpgSystemRegistry {
    systems: HashMap<String, RpgSystem>,
}

impl RpgSystemRegistry {
    pub fn load(systems_directory: &Path) -> anyhow::Result<Self> {
        let mut systems = HashMap::new();

        if !systems_directory.exists() {
            return Ok(Self { systems });
        }

        let entries = std::fs::read_dir(systems_directory).with_context(|| {
            format!("Failed to read rpg-systems directory: {systems_directory:?}")
        })?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if is_yaml_file(&path) {
                let system = load_system_from_file(&path)?;
                systems.insert(system.id.0.clone(), system);
            }
        }

        Ok(Self { systems })
    }

    pub fn get(&self, id: &str) -> Option<&RpgSystem> {
        self.systems.get(id)
    }

    pub fn list_all(&self) -> Vec<&RpgSystem> {
        self.systems.values().collect()
    }
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "yaml" || ext == "yml")
        .unwrap_or(false)
}

fn load_system_from_file(path: &Path) -> anyhow::Result<RpgSystem> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read rpg system file: {path:?}"))?;

    serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse rpg system YAML from: {path:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_yaml_file(directory: &Path, filename: &str, content: &str) {
        let path = directory.join(filename);
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_returns_empty_registry_when_directory_does_not_exist() {
        let registry = RpgSystemRegistry::load(Path::new("/nonexistent/path")).unwrap();
        assert_eq!(registry.list_all().len(), 0);
    }

    #[test]
    fn get_returns_none_for_missing_system() {
        let temp_directory = TempDir::new().unwrap();
        let registry = RpgSystemRegistry::load(temp_directory.path()).unwrap();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn non_yaml_files_are_ignored() {
        let temp_directory = TempDir::new().unwrap();
        write_yaml_file(temp_directory.path(), "readme.txt", "not yaml");

        let registry = RpgSystemRegistry::load(temp_directory.path()).unwrap();
        assert_eq!(registry.list_all().len(), 0);
    }
}
