use std::error::Error;
use tauri::Window;

pub const OUTPUT_FILE_BASE_PATH: &str = "../data/";

pub async fn save_text(title: &str, japanese: &str, english: &str, file_prefix: &str, window: &Window) -> Result<String, Box<dyn Error>> {
    println!("save_text");

    window.emit("add_to_output", Some("
# SaveText")).expect("failed to emit event");

    let file_name = format!("{}{}.txt", OUTPUT_FILE_BASE_PATH, file_prefix);
    let text = format!("# Title
{}

# Japanese
{}

# English
{}", title, japanese, english);
    std::fs::write(file_name.clone(), text).expect("failed to write file");

    window.emit("add_to_output", Some(format!("FilePath: {}", file_name))).expect("failed to emit event");

    Ok(file_name.to_string())
}

pub fn ensure_directory_exists() {
    std::fs::create_dir_all(OUTPUT_FILE_BASE_PATH).expect("failed to create directory");
}
