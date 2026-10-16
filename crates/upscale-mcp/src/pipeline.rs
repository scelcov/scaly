use std::path::{Path, PathBuf};
use std::process::Command;

use upscale_core::{Error, MediaKind, UpscaleRequest};

/// Run a GPU upscale with ffmpeg libplacebo. `request.scale` is 2, 3, or 4.
///
/// # Errors
///
/// Returns [`Error::InputNotFound`] when the source is missing, [`Error::SamePath`]
/// when the output path is the input, and [`Error::Upscale`] when ffmpeg fails.
pub fn upscale(request: &UpscaleRequest) -> Result<(), Error> {
    let input = absolute(&request.input_path)?;
    let output = absolute(&request.output_path)?;
    if !input.is_file() {
        return Err(Error::InputNotFound);
    }
    if input == output {
        return Err(Error::SamePath);
    }
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|err| Error::Upscale(format!("failed to create output directory: {err}")))?;
    }

    let mut command = Command::new("ffmpeg");
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(&input);
    match request.kind {
        MediaKind::Image => {
            command
                .arg("-vf")
                .arg(filter(request.scale, false))
                .arg("-frames:v")
                .arg("1")
                .arg("-update")
                .arg("1")
                .arg(&output);
        }
        MediaKind::Video => {
            command
                .arg("-vf")
                .arg(filter(request.scale, true))
                .arg("-map")
                .arg("0:v:0")
                .arg("-map")
                .arg("0:a?")
                .arg("-c:v")
                .arg("libx264")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-crf")
                .arg("18")
                .arg("-preset")
                .arg("veryfast")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("192k")
                .arg(&output);
        }
    }
    run(&mut command)?;
    let meta = std::fs::metadata(&output)
        .map_err(|err| Error::Upscale(format!("output was not written: {err}")))?;
    if meta.len() == 0 {
        return Err(Error::Upscale("output file is empty".to_owned()));
    }
    Ok(())
}

/// First frame of a finished video, as JPEG bytes for the chat preview.
///
/// # Errors
///
/// Returns [`Error::Upscale`] when ffmpeg cannot read the file.
pub fn poster_jpeg(video: &Path) -> Result<Vec<u8>, Error> {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(video)
        .arg("-frames:v")
        .arg("1")
        .arg("-f")
        .arg("image2pipe")
        .arg("-vcodec")
        .arg("mjpeg")
        .arg("pipe:1")
        .output()
        .map_err(|err| Error::Upscale(format!("failed to start ffmpeg: {err}")))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(Error::Upscale(ffmpeg_tail(&output.stderr, output.status)));
    }
    Ok(output.stdout)
}

fn filter(scale: u8, deband: bool) -> String {
    let mut graph = format!("libplacebo=w=iw*{scale}:h=ih*{scale}:upscaler=ewa_lanczossharp");
    if deband {
        graph.push_str(":deband=1");
    }
    graph
}

fn absolute(path: &Path) -> Result<PathBuf, Error> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir()
        .map_err(|err| Error::Upscale(format!("failed to read working directory: {err}")))?;
    Ok(cwd.join(path))
}

fn run(command: &mut Command) -> Result<(), Error> {
    let output = command
        .output()
        .map_err(|err| Error::Upscale(format!("failed to start ffmpeg: {err}")))?;
    if output.status.success() {
        return Ok(());
    }
    Err(Error::Upscale(ffmpeg_tail(&output.stderr, output.status)))
}

fn ffmpeg_tail(stderr: &[u8], status: std::process::ExitStatus) -> String {
    let text = String::from_utf8_lossy(stderr);
    let tail: String = text.chars().rev().take(400).collect();
    let tail: String = tail.chars().rev().collect();
    let tail = tail.trim();
    if tail.is_empty() {
        format!("ffmpeg exited {status}")
    } else {
        format!("ffmpeg exited {status}: {tail}")
    }
}
