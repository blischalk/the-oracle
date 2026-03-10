use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;

use crate::domain::rpg_system::RpgSystem;

pub struct RpgSystemRegistry {
    systems: HashMap<String, RpgSystem>,
}

impl RpgSystemRegistry {
    /// Loads systems from a single directory. Returns an empty registry if the
    /// directory does not exist.
    pub fn load(systems_directory: &Path) -> anyhow::Result<Self> {
        Self::load_from_directories(&[systems_directory])
    }

    /// Loads systems from multiple directories in order. Later directories
    /// override systems with the same id from earlier directories, so user
    /// systems can shadow bundled ones.
    pub fn load_from_directories(directories: &[&Path]) -> anyhow::Result<Self> {
        let mut systems = HashMap::new();

        for directory in directories {
            if !directory.exists() {
                continue;
            }

            let entries = std::fs::read_dir(directory).with_context(|| {
                format!("Failed to read rpg-systems directory: {directory:?}")
            })?;

            for entry in entries {
                let path = entry.context("Failed to read directory entry")?.path();

                if !is_yaml_file(&path) {
                    continue;
                }

                match load_system_from_file(&path) {
                    Ok(system) => {
                        systems.insert(system.id.0.clone(), system);
                    }
                    Err(error) => {
                        eprintln!("Warning: skipping {path:?}: {error}");
                    }
                }
            }
        }

        Ok(Self { systems })
    }

    pub fn get(&self, id: &str) -> Option<&RpgSystem> {
        self.systems.get(id)
    }

    pub fn list_all(&self) -> Vec<&RpgSystem> {
        let mut list: Vec<&RpgSystem> = self.systems.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
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

    const MINIMAL_YAML: &str = r#"
id: test-system
name: Test System
system_prompt: "You are a GM."
character_fields: []
"#;

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

    #[test]
    fn loads_valid_yaml_file() {
        let temp_directory = TempDir::new().unwrap();
        write_yaml_file(temp_directory.path(), "test.yaml", MINIMAL_YAML);

        let registry = RpgSystemRegistry::load(temp_directory.path()).unwrap();
        assert_eq!(registry.list_all().len(), 1);
        assert!(registry.get("test-system").is_some());
    }

    #[test]
    fn user_directory_overrides_bundled_system_with_same_id() {
        let bundled = TempDir::new().unwrap();
        let user = TempDir::new().unwrap();

        write_yaml_file(bundled.path(), "system.yaml", MINIMAL_YAML);
        write_yaml_file(
            user.path(),
            "system.yaml",
            r#"
id: test-system
name: User Override
system_prompt: "Custom GM."
character_fields: []
"#,
        );

        let registry = RpgSystemRegistry::load_from_directories(&[
            bundled.path(),
            user.path(),
        ])
        .unwrap();

        assert_eq!(registry.list_all().len(), 1);
        assert_eq!(registry.get("test-system").unwrap().name, "User Override");
    }

    #[test]
    fn invalid_yaml_file_is_skipped_without_aborting() {
        let temp_directory = TempDir::new().unwrap();
        write_yaml_file(temp_directory.path(), "bad.yaml", "not: valid: yaml: {{{");
        write_yaml_file(temp_directory.path(), "good.yaml", MINIMAL_YAML);

        let registry = RpgSystemRegistry::load(temp_directory.path()).unwrap();
        assert_eq!(registry.list_all().len(), 1);
    }

    #[test]
    fn list_all_returns_systems_sorted_by_name() {
        let temp_directory = TempDir::new().unwrap();
        write_yaml_file(
            temp_directory.path(),
            "b.yaml",
            "id: b\nname: Zebra System\nsystem_prompt: GM\ncharacter_fields: []",
        );
        write_yaml_file(
            temp_directory.path(),
            "a.yaml",
            "id: a\nname: Alpha System\nsystem_prompt: GM\ncharacter_fields: []",
        );

        let registry = RpgSystemRegistry::load(temp_directory.path()).unwrap();
        let names: Vec<&str> = registry.list_all().iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha System", "Zebra System"]);
    }

    #[test]
    fn bundled_rpg_systems_all_parse_successfully() {
        let bundled_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("rpg-systems");

        if !bundled_dir.exists() {
            println!("rpg-systems directory not found at {bundled_dir:?}, skipping");
            return;
        }

        for entry in std::fs::read_dir(&bundled_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                let content = std::fs::read_to_string(&path).unwrap();
                let result: Result<crate::domain::rpg_system::RpgSystem, _> =
                    serde_yaml::from_str(&content);
                assert!(
                    result.is_ok(),
                    "Failed to parse {:?}: {}",
                    path.file_name().unwrap(),
                    result.unwrap_err()
                );
            }
        }
    }
}
