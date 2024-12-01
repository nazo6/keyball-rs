use std::process::Command;

fn main() -> Result<(), String> {
    let args = std::env::args().collect::<Vec<_>>();
    let saved_file = args
        .get(1)
        .ok_or_else(|| "No saved file path provided".to_string())?;

    let mut path = std::path::PathBuf::from(saved_file);

    loop {
        path = path
            .parent()
            .ok_or_else(|| format!("Could not find Cargo.toml for file: {:?}", saved_file))?
            .to_owned();

        if !path.join("Cargo.toml").exists() {
            continue;
        } else {
            break;
        }
    }

    let mut child = Command::new("cargo")
        .args(["clippy", "--all-features", "--message-format=json"])
        .current_dir(path)
        .spawn()
        .map_err(|e| format!("Error running cargo clippy: {:?}", e))?;
    child
        .wait()
        .map_err(|e| format!("Error waiting for cargo clippy: {:?}", e))?;

    Ok(())
}
