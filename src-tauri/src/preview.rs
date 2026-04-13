//! Local HTTP preview server and media-opening commands.
//!
//! The desktop webview can preview media through localhost URLs. This module
//! owns token registration, byte-range serving, and opening media in the
//! system player.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tauri::State;

use crate::AppState;

#[derive(Clone)]
pub(crate) struct MediaPreviewServer {
    base_url: String,
    routes: Arc<Mutex<HashMap<String, PathBuf>>>,
    next_token: Arc<AtomicU64>,
}

impl MediaPreviewServer {
    fn register(&self, path: PathBuf) -> Result<String, String> {
        let token = format!("{:x}", self.next_token.fetch_add(1, Ordering::SeqCst));
        let mut routes = self
            .routes
            .lock()
            .map_err(|_| "Media preview server route registry is unavailable.".to_string())?;
        routes.insert(token.clone(), path);
        Ok(format!("{}/media/{}", self.base_url, token))
    }
}

pub(crate) fn start_media_preview_server() -> Result<MediaPreviewServer, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("Failed to start media preview server: {error}"))?;
    let addr = listener
        .local_addr()
        .map_err(|error| format!("Failed to read media preview server address: {error}"))?;
    let routes = Arc::new(Mutex::new(HashMap::<String, PathBuf>::new()));
    let server_routes = Arc::clone(&routes);

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let routes = Arc::clone(&server_routes);
            thread::spawn(move || handle_media_preview_request(stream, routes));
        }
    });

    Ok(MediaPreviewServer {
        base_url: format!("http://{addr}"),
        routes,
        next_token: Arc::new(AtomicU64::new(1)),
    })
}

fn handle_media_preview_request(
    mut stream: TcpStream,
    routes: Arc<Mutex<HashMap<String, PathBuf>>>,
) {
    if let Err(error) = serve_media_preview_request(&mut stream, routes) {
        let _ = write_http_response(
            &mut stream,
            "500 Internal Server Error",
            "text/plain",
            &[("Cache-Control", "no-store")],
            error.as_bytes(),
        );
    }
}

fn serve_media_preview_request(
    stream: &mut TcpStream,
    routes: Arc<Mutex<HashMap<String, PathBuf>>>,
) -> Result<(), String> {
    let reader_stream = stream
        .try_clone()
        .map_err(|error| format!("Failed to read preview request: {error}"))?;
    let mut reader = BufReader::new(reader_stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|error| format!("Failed to read preview request line: {error}"))?;

    let mut headers = HashMap::<String, String>::new();
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| format!("Failed to read preview request headers: {error}"))?;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default();
    let path = request_parts.next().unwrap_or_default();

    if method == "OPTIONS" {
        return write_http_response(
            stream,
            "204 No Content",
            "text/plain",
            &[("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS")],
            &[],
        )
        .map_err(|error| format!("Failed to write preview response: {error}"));
    }

    if method != "GET" && method != "HEAD" {
        return write_http_response(
            stream,
            "405 Method Not Allowed",
            "text/plain",
            &[("Allow", "GET, HEAD, OPTIONS")],
            b"Method not allowed",
        )
        .map_err(|error| format!("Failed to write preview response: {error}"));
    }

    let token = path
        .strip_prefix("/media/")
        .and_then(|value| value.split(['?', '#']).next())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Invalid preview URL.".to_string())?;
    let media_path = {
        let routes = routes
            .lock()
            .map_err(|_| "Media preview server route registry is unavailable.".to_string())?;
        routes
            .get(token)
            .cloned()
            .ok_or_else(|| "Media preview URL is no longer registered.".to_string())?
    };

    serve_media_file(stream, method == "HEAD", &media_path, headers.get("range"))
}

fn serve_media_file(
    stream: &mut TcpStream,
    head_only: bool,
    path: &Path,
    range_header: Option<&String>,
) -> Result<(), String> {
    let mut file =
        File::open(path).map_err(|error| format!("Failed to open preview media file: {error}"))?;
    let file_size = file
        .metadata()
        .map_err(|error| format!("Failed to read preview media metadata: {error}"))?
        .len();

    let (status, start, end) = match parse_range_header(range_header, file_size) {
        Some((start, end)) => ("206 Partial Content", start, end),
        None => ("200 OK", 0, file_size.saturating_sub(1)),
    };
    let content_length = if file_size == 0 { 0 } else { end - start + 1 };

    let mut response_headers = vec![
        ("Accept-Ranges".to_string(), "bytes".to_string()),
        ("Cache-Control".to_string(), "no-store".to_string()),
        ("Content-Length".to_string(), content_length.to_string()),
    ];
    if status.starts_with("206") {
        response_headers.push((
            "Content-Range".to_string(),
            format!("bytes {start}-{end}/{file_size}"),
        ));
    }

    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\n",
        media_content_type(path)
    )
    .map_err(|error| format!("Failed to write preview response: {error}"))?;
    for (key, value) in response_headers {
        write!(stream, "{key}: {value}\r\n")
            .map_err(|error| format!("Failed to write preview response header: {error}"))?;
    }
    write!(stream, "\r\n")
        .map_err(|error| format!("Failed to finish preview response headers: {error}"))?;

    if head_only || content_length == 0 {
        return Ok(());
    }

    file.seek(SeekFrom::Start(start))
        .map_err(|error| format!("Failed to seek preview media file: {error}"))?;
    let mut remaining = content_length;
    let mut buffer = [0u8; 64 * 1024];
    while remaining > 0 {
        let read_limit = remaining.min(buffer.len() as u64) as usize;
        let bytes_read = file
            .read(&mut buffer[..read_limit])
            .map_err(|error| format!("Failed to read preview media file: {error}"))?;
        if bytes_read == 0 {
            break;
        }
        stream
            .write_all(&buffer[..bytes_read])
            .map_err(|error| format!("Failed to stream preview media file: {error}"))?;
        remaining -= bytes_read as u64;
    }

    Ok(())
}

fn write_http_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> std::io::Result<()> {
    write!(
    stream,
    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n",
    body.len()
  )?;
    for (key, value) in headers {
        write!(stream, "{key}: {value}\r\n")?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(body)
}

fn parse_range_header(range_header: Option<&String>, file_size: u64) -> Option<(u64, u64)> {
    if file_size == 0 {
        return None;
    }

    let range = range_header?.strip_prefix("bytes=")?;
    let (start, end) = range.split_once('-')?;
    let start = if start.is_empty() {
        let suffix_length = end.parse::<u64>().ok()?;
        file_size.saturating_sub(suffix_length)
    } else {
        start.parse::<u64>().ok()?
    };
    let end = if end.is_empty() {
        file_size - 1
    } else {
        end.parse::<u64>().ok()?.min(file_size - 1)
    };

    (start <= end && start < file_size).then_some((start, end))
}

fn media_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
pub(crate) fn open_media_in_system_player(path: String) -> Result<(), String> {
    let input = PathBuf::from(&path);
    if !input.exists() {
        return Err("Video file does not exist.".into());
    }

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(&path);
        command
    };

    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(&path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", &path]);
        command
    };

    command
        .spawn()
        .map_err(|error| format!("Failed to open system video player: {error}"))?;

    Ok(())
}

#[tauri::command]
pub(crate) fn get_media_preview_url(
    state: State<AppState>,
    path: String,
) -> Result<String, String> {
    let input = PathBuf::from(path);
    if !input.is_file() {
        return Err("Preview media file does not exist.".into());
    }

    let canonical_path = input
        .canonicalize()
        .map_err(|error| format!("Failed to resolve preview media path: {error}"))?;
    state.preview_server.register(canonical_path)
}
