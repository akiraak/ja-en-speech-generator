use std::error::Error;
use tauri::Window;
use crate::utils::{chatgpt, file, audio};

#[tauri::command]
pub fn submit_command(command: &str, window: Window) -> String {
    println!("submit_command: {}", command);
    let command_owned = command.to_string();

    tokio::spawn(async move {
        if let Err(e) = execute_workflow(command_owned, window).await {
            eprintln!("Failed to execute workflow: {}", e);
        }
    });

    format!("Command execution started for: {}", command)
}

async fn execute_workflow(command: String, window: Window) -> Result<(), Box<dyn Error>> {
    println!("execute_workflow");

    let file_prefix = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();

    let (ja_text, ja_mp3_path) = chatgpt::make_japanese(&command, &file_prefix, &window).await?;
    let (en_text, en_mp3_path) = chatgpt::make_english(&ja_text, &file_prefix, &window).await?;
    file::save_text(&command, &ja_text, &en_text, &file_prefix, &window).await?;

    let merged_mp3_output_path = format!("{}{}.mp3", file::OUTPUT_FILE_BASE_PATH, file_prefix);
    audio::marge_mp3_files(&[
            &ja_mp3_path,
            "../silence_1sec.mp3",
            &en_mp3_path
        ], &merged_mp3_output_path)
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    window.emit("add_to_output", Some(format!("
# MargeMP3
FilePath: {}", merged_mp3_output_path))).expect("failed to emit event");

    audio::play_mp3(&merged_mp3_output_path)?;

    window.emit("add_to_output", Some(format!("
Done!"))).expect("failed to emit event");

    Ok(())
}
