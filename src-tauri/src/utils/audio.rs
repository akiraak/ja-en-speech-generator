use rodio::{Decoder, OutputStream, Sink, Source};
use std::{fs::File, io::BufReader, process::Command, error::Error};
use std::io::Write;

pub fn marge_mp3_files(files: &[&str], output_file: &str) -> std::io::Result<()> {
    // リストファイルを作成
    let mut list_file = File::create("../data/marge_mp3_list.txt")?;
    for &mp3_file in files {
        writeln!(list_file, "file '{}'", mp3_file)?;
    }

    // FFmpeg コマンドを実行してファイルを結合
    let status = Command::new("ffmpeg")
        .args(&["-y", "-f", "concat", "-safe", "0", "-i", "../data/marge_mp3_list.txt", "-c", "copy", output_file])
        .status()?;

    if status.success() {
        println!("MP3 files were concatenated successfully.");
    } else {
        eprintln!("Failed to concatenate MP3 files.");
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to concatenate MP3 files"));
    }

    Ok(())
}

pub fn play_mp3(file_path: &str) -> Result<(), Box<dyn Error>> {
    println!("play_mp3");

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let file = BufReader::new(File::open(file_path)?);
    let source = Decoder::new(file)?.convert_samples::<f32>();

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
