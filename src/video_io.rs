use anyhow::{Result, anyhow};
use image::DynamicImage;
use std::path::{Path, PathBuf};
use ffmpeg_sidecar::command::FfmpegCommand;
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration: f32,
}

pub fn get_metadata(path: &Path) -> Result<VideoMetadata> {
    log::info!("video_io: Probing metadata for {:?}", path);
    // Use absolute path for ffprobe
    let mut command = std::process::Command::new(r"C:\ffmpeg\bin\ffprobe.exe");
    command.args([
        "-v", "quiet",
        "-print_format", "json",
        "-show_streams",
        "-show_format",
        path.to_str().unwrap()
    ]);

    let output = command.output()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let stream = json["streams"]
        .as_array()
        .and_then(|streams| streams.iter().find(|s| s["codec_type"] == "video"))
        .ok_or_else(|| anyhow!("No video stream found"))?;

    let width = stream["width"].as_u64().unwrap_or(0) as u32;
    let height = stream["height"].as_u64().unwrap_or(0) as u32;
    
    // Parse "30/1" or "60000/1001" FPS strings
    let r_frame_rate = stream["r_frame_rate"].as_str().unwrap_or("24/1");
    let fps = if let Some((num, den)) = r_frame_rate.split_once('/') {
        let n: f32 = num.parse().unwrap_or(24.0);
        let d: f32 = den.parse().unwrap_or(1.0);
        if d != 0.0 { n / d } else { 24.0 }
    } else {
        24.0
    };

    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);

    Ok(VideoMetadata { width, height, fps, duration })
}

pub struct VideoStream {
    pub receiver: Receiver<Vec<u8>>,
    pub metadata: VideoMetadata,
}

pub fn spawn_video_stream(path: PathBuf, start_time: f32, existing_metadata: Option<VideoMetadata>) -> Result<VideoStream> {
    let metadata = if let Some(m) = existing_metadata {
        m
    } else {
        log::info!("video_io: Probing metadata for stream...");
        get_metadata(&path)?
    };

    log::info!("video_io: Spawning stream for {:?} ({}x{} @ {} FPS) starting at {}s", path, metadata.width, metadata.height, metadata.fps, start_time);
    
    let (tx, rx) = mpsc::sync_channel(20); 

    thread::spawn(move || {
        let mut command = FfmpegCommand::new_with_path(r"C:\ffmpeg\bin\ffmpeg.exe");
        
        // Input options (loop and seek) MUST come before .input()
        command.args(["-stream_loop", "-1"]);
        if start_time > 0.0 {
            command.args(["-ss", &start_time.to_string()]);
        }

        command
            .input(path.to_str().unwrap())
            .format("rawvideo")
            .pix_fmt("rgba")
            .args(["-"])
            .print_command();

        if let Ok(mut child) = command.spawn() {
            if let Ok(iter) = child.iter() {
                for frame in iter.filter_frames() {
                    // Send the raw Vec<u8> directly
                    if tx.send(frame.data).is_err() {
                        break; 
                    }
                }
            }
        }
    });

    Ok(VideoStream { receiver: rx, metadata })
}

pub fn load_first_frame(path: &Path) -> Result<DynamicImage> {
    log::info!("video_io: Loading first frame from {:?}", path);

    let mut command = FfmpegCommand::new_with_path(r"C:\ffmpeg\bin\ffmpeg.exe");
    command
        .input(path.to_str().ok_or_else(|| anyhow!("Invalid path"))?)
        .args(["-f", "image2pipe", "-vcodec", "png", "-frames:v", "1", "-"])
        .print_command();

    let mut child = command.spawn()?;
    let mut stdout = child.take_stdout().ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut stdout, &mut buffer)?;

    if buffer.is_empty() {
        return Err(anyhow!("video_io: Extracted frame data is empty"));
    }

    let img = image::load_from_memory(&buffer)?;
    Ok(img)
}
