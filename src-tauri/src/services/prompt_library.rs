use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;

pub struct PromptLibrary {
    prompts: HashMap<String, String>,
}

impl PromptLibrary {
    /// Loads all .md files from the prompts directory and its immediate subdirectories.
    /// Returns an empty library if the directory does not exist.
    pub fn load(directory: &Path) -> anyhow::Result<Self> {
        let mut prompts = HashMap::new();

        if !directory.exists() {
            return Ok(Self { prompts });
        }

        let entries = std::fs::read_dir(directory)
            .with_context(|| format!("Failed to read prompts directory: {directory:?}"))?;

        for entry in entries {
            let path = entry.context("Failed to read directory entry")?.path();

            if path.is_dir() {
                let prefix = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let sub_entries = std::fs::read_dir(&path)
                    .with_context(|| format!("Failed to read prompts subdirectory: {path:?}"))?;

                for sub_entry in sub_entries {
                    let sub_path = sub_entry.context("Failed to read directory entry")?.path();
                    if is_markdown_file(&sub_path) {
                        let key = derive_key(&sub_path, Some(&prefix));
                        match load_prompt_from_file(&sub_path) {
                            Ok(content) => {
                                prompts.insert(key, content);
                            }
                            Err(error) => {
                                eprintln!("Warning: skipping {sub_path:?}: {error}");
                            }
                        }
                    }
                }
            } else if is_markdown_file(&path) {
                let key = derive_key(&path, None);
                match load_prompt_from_file(&path) {
                    Ok(content) => {
                        prompts.insert(key, content);
                    }
                    Err(error) => {
                        eprintln!("Warning: skipping {path:?}: {error}");
                    }
                }
            }
        }

        Ok(Self { prompts })
    }

    /// Returns an empty library. Used in tests.
    pub fn empty() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }

    /// Returns the prompt text for a given key (e.g. "system/mechanics", "tasks/greeting_new_campaign").
    pub fn get(&self, key: &str) -> Option<&str> {
        self.prompts.get(key).map(String::as_str)
    }

    /// Returns the prompt text with all `{{token}}` occurrences replaced.
    pub fn render(&self, key: &str, substitutions: &[(&str, &str)]) -> Option<String> {
        let mut text = self.get(key)?.to_string();
        for (token, value) in substitutions {
            text = text.replace(&format!("{{{{{token}}}}}"), value);
        }
        Some(text)
    }
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md")
        .unwrap_or(false)
}

fn derive_key(path: &Path, prefix: Option<&str>) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match prefix {
        Some(p) => format!("{p}/{stem}"),
        None => stem.to_string(),
    }
}

fn load_prompt_from_file(path: &Path) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read prompt file: {path:?}"))?;
    Ok(content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(directory: &Path, filename: &str, content: &str) {
        let path = directory.join(filename);
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    fn write_in_subdir(parent: &Path, subdir: &str, filename: &str, content: &str) {
        let dir = parent.join(subdir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(filename);
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn load_returns_empty_library_when_directory_does_not_exist() {
        let library = PromptLibrary::load(Path::new("/nonexistent/path")).unwrap();
        assert!(library.get("anything").is_none());
    }

    #[test]
    fn empty_returns_empty_library() {
        let library = PromptLibrary::empty();
        assert!(library.get("anything").is_none());
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let temp = TempDir::new().unwrap();
        let library = PromptLibrary::load(temp.path()).unwrap();
        assert!(library.get("system/missing").is_none());
    }

    #[test]
    fn loads_markdown_file_from_subdirectory() {
        let temp = TempDir::new().unwrap();
        write_in_subdir(temp.path(), "system", "mechanics.md", "Use the dice tools.");

        let library = PromptLibrary::load(temp.path()).unwrap();
        assert_eq!(library.get("system/mechanics"), Some("Use the dice tools."));
    }

    #[test]
    fn loads_markdown_file_from_root() {
        let temp = TempDir::new().unwrap();
        write_file(temp.path(), "readme.md", "Top level prompt.");

        let library = PromptLibrary::load(temp.path()).unwrap();
        assert_eq!(library.get("readme"), Some("Top level prompt."));
    }

    #[test]
    fn non_markdown_files_are_ignored() {
        let temp = TempDir::new().unwrap();
        write_in_subdir(temp.path(), "system", "notes.txt", "not a prompt");

        let library = PromptLibrary::load(temp.path()).unwrap();
        assert!(library.get("system/notes").is_none());
    }

    #[test]
    fn file_content_is_trimmed() {
        let temp = TempDir::new().unwrap();
        write_in_subdir(temp.path(), "system", "role.md", "\n  Trimmed content.  \n");

        let library = PromptLibrary::load(temp.path()).unwrap();
        assert_eq!(library.get("system/role"), Some("Trimmed content."));
    }

    #[test]
    fn render_substitutes_tokens() {
        let temp = TempDir::new().unwrap();
        write_in_subdir(
            temp.path(),
            "tasks",
            "extract.md",
            "Fields: {{field_list}}. Conversation:\n{{conversation}}",
        );

        let library = PromptLibrary::load(temp.path()).unwrap();
        let result = library
            .render(
                "tasks/extract",
                &[("field_list", "str, dex"), ("conversation", "Player: hello")],
            )
            .unwrap();

        assert_eq!(result, "Fields: str, dex. Conversation:\nPlayer: hello");
    }

    #[test]
    fn render_returns_none_for_missing_key() {
        let library = PromptLibrary::empty();
        assert!(library.render("missing/key", &[]).is_none());
    }

    #[test]
    fn bundled_prompts_all_load_successfully() {
        let prompts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("prompts");

        if !prompts_dir.exists() {
            println!("prompts directory not found at {prompts_dir:?}, skipping");
            return;
        }

        let library = PromptLibrary::load(&prompts_dir).unwrap();
        for key in [
            "system/role",
            "system/forbidden",
            "system/formatting",
            "system/pacing",
            "system/continuity",
            "system/mechanics",
            "system/character_creation",
            "system/confrontation",
            "system/naming",
            "system/creation_alert",
            "system/creation_reminder",
            "tasks/suggest_campaign_name",
            "tasks/extract_character_data",
            "tasks/greeting_new_campaign",
            "tasks/greeting_resume_campaign",
        ] {
            assert!(
                library.get(key).is_some(),
                "Expected prompt key '{key}' to be present but it was missing"
            );
        }
    }
}
