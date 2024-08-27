use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    path::Path,
};

pub fn write_to_file(path: &str, content: &str) -> std::io::Result<()> {
    // Create all parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Open the file in write mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    // Write the content to the file
    writeln!(file, "{}", content)?;

    Ok(())
}
