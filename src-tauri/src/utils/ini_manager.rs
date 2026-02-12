use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write};

pub struct IniManager {
    path: PathBuf,
    content: String,
}

impl IniManager {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        println!("[IniManager] Loading ini from: {:?}", path);
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read ini file: {}", e))?;

        Ok(Self { path, content })
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        println!("[IniManager] Setting [{}] {} = {}", section, key, value);
        // Simple state machine parser to handle sections and keys while preserving structure
        let mut lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut in_target_section = false;
        let mut key_found = false;
        let mut target_section_index = -1;
        let mut last_line_of_section = -1;

        let section_lower = section.to_lowercase();

        for (i, line) in lines.iter_mut().enumerate() {
            let trimmed = line.trim();
            // Remove inline comments for checking section start
            let clean_line = trimmed.split(';').next().unwrap_or("").trim();

            if clean_line.starts_with('[') && clean_line.ends_with(']') {
                let current_section = clean_line[1..clean_line.len() - 1].trim().to_lowercase();

                println!(
                    "[IniManager] Found section: [{}] at line {}",
                    current_section, i
                );

                if current_section == section_lower {
                    println!(
                        "[IniManager] Entering target section: [{}]",
                        current_section
                    );
                    in_target_section = true;
                    target_section_index = i as i32;
                } else {
                    if in_target_section {
                        // We were in the section, now we are leaving it
                        println!("[IniManager] Leaving target section at line {}", i);
                        last_line_of_section = (i as i32) - 1;
                        in_target_section = false;
                    }
                }
                continue;
            }

            if in_target_section {
                // Check if line is a key
                // Ignore comments
                if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() {
                    continue;
                }

                if let Some(eq_idx) = line.find('=') {
                    let current_key = line[..eq_idx].trim();
                    if current_key.eq_ignore_ascii_case(key) {
                        println!(
                            "[IniManager] Found key '{}' at line {}. Updating value.",
                            current_key, i
                        );
                        // Found key, replace value
                        // Keep indentation of key if possible ? For now simple replace
                        // We try to preserve the part before '=' to keep indentation
                        let prefix = &line[..eq_idx];
                        *line = format!("{} = {}", prefix.trim_end(), value);
                        key_found = true;
                        break;
                    }
                }
            }
        }

        // Handle cases where key was not found
        if !key_found {
            println!("[IniManager] Key '{}' not found. Inserting.", key);
            if target_section_index != -1 {
                // Section exists, append key
                // Find where to insert: end of section or before next section
                // If we are still "in_target_section" at the end of loop, last line is EOF

                let insert_pos = if last_line_of_section != -1 {
                    // Start searching backwards from next section start to find last non-empty line?
                    // Or just insert before next section.
                    // Inserting before next section header (which is at last_line_of_section + 1)
                    (last_line_of_section + 1) as usize
                } else if in_target_section {
                    // Section goes until EOF
                    lines.len()
                } else {
                    // This case means we found section but then lost it?
                    // Handled by last_line_of_section logic above.
                    // But if section is strictly at the end?
                    // Re-scan to find true end
                    let mut pos = lines.len();
                    for (i, line) in lines
                        .iter()
                        .enumerate()
                        .skip((target_section_index + 1) as usize)
                    {
                        let trimmed = line.trim();
                        let clean_line = trimmed.split(';').next().unwrap_or("").trim();
                        if clean_line.starts_with('[') {
                            pos = i;
                            break;
                        }
                    }
                    pos
                };

                println!("[IniManager] Inserting at line {}", insert_pos);
                lines.insert(insert_pos, format!("{} = {}", key, value));
            } else {
                // Section does not exist, valid INI creates it?
                // User requirement: "if no [Loader], add line? but cannot go to other section"
                // Implies we expect section to exist.
                // Creating section at EOF is safe fallback.
                println!(
                    "[IniManager] Section [{}] not found. Creating at EOF.",
                    section
                );
                lines.push(String::new());
                lines.push(format!("[{}]", section));
                lines.push(format!("{} = {}", key, value));
            }
        }

        self.content = lines.join("\n"); // Using \n for consistency
    }

    pub fn remove_key(&mut self, section: &str, key: &str) {
        println!("[IniManager] Removing key [{}] {}", section, key);
        let mut lines: Vec<String> = self.content.lines().map(|s| s.to_string()).collect();
        let mut in_target_section = false;
        let mut lines_to_remove = Vec::new();

        let section_lower = section.to_lowercase();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let clean_line = trimmed.split(';').next().unwrap_or("").trim();

            if clean_line.starts_with('[') && clean_line.ends_with(']') {
                let current_section = clean_line[1..clean_line.len() - 1].trim().to_lowercase();
                if current_section == section_lower {
                    in_target_section = true;
                } else {
                    in_target_section = false;
                }
                continue;
            }

            if in_target_section {
                if trimmed.starts_with(';') || trimmed.starts_with('#') || trimmed.is_empty() {
                    continue;
                }
                if let Some(eq_idx) = line.find('=') {
                    let current_key = line[..eq_idx].trim();
                    if current_key.eq_ignore_ascii_case(key) {
                        lines_to_remove.push(i);
                    }
                }
            }
        }

        // Remove in reverse order
        for i in lines_to_remove.into_iter().rev() {
            lines.remove(i);
        }

        self.content = lines.join("\n");
    }

    pub fn save(&self) -> Result<(), String> {
        println!("[IniManager] Saving ini to: {:?}", self.path);
        let mut file = fs::File::create(&self.path)
            .map_err(|e| format!("Failed to open file for writing: {}", e))?;
        file.write_all(self.content.as_bytes())
            .map_err(|e| format!("Failed to write content: {}", e))?;
        println!(
            "[IniManager] Successfully wrote {} bytes.",
            self.content.len()
        );
        Ok(())
    }
}
