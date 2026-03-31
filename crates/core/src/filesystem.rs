use std::fs;
use std::io;
use std::path::Path;

pub fn ensure_directory(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

pub fn write_text_atomic(path: &Path, contents: &str) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "cannot write file without parent directory: {}",
                path.display()
            ),
        )
    })?;
    let temp_name = format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("zazz-temp")
    );
    let temp_path = parent.join(temp_name);

    ensure_directory(parent)?;
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn ensure_file_contains_line(path: &Path, required_line: &str) -> io::Result<()> {
    let existing = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let already_present = existing.lines().any(|line| line == required_line);
    if already_present {
        return Ok(());
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(required_line);
    updated.push('\n');
    write_text_atomic(path, &updated)
}

pub fn copy_file(source: &Path, destination: &Path) -> io::Result<u64> {
    if let Some(parent) = destination.parent() {
        ensure_directory(parent)?;
    }
    fs::copy(source, destination)
}

pub fn copy_directory_recursive(source: &Path, destination: &Path) -> io::Result<()> {
    ensure_directory(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if entry.file_name() == ".git" {
                continue;
            }
            copy_directory_recursive(&entry_path, &target_path)?;
        } else if file_type.is_file() {
            copy_file(&entry_path, &target_path)?;
        }
    }
    Ok(())
}
