use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

use crate::model::page_file_name;

const RELOAD_PATH: &str = "/__siteforge/reload";
const RELOAD_SNIPPET: &str = r#"<script>
(function(){
  var last=null;
  function tick(){
    fetch("/__siteforge/reload",{cache:"no-store"}).then(function(r){return r.text();}).then(function(t){
      if(last===null) last=t;
      else if(t!==last) location.reload();
    }).catch(function(){});
    setTimeout(tick,1000);
  }
  tick();
})();
</script>"#;

pub struct StaticServer {
    pub port: u16,
    root: Arc<Mutex<PathBuf>>,
    generation: Arc<AtomicU64>,
}

impl StaticServer {
    pub fn start(root: PathBuf) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let root = Arc::new(Mutex::new(root));
        let generation = Arc::new(AtomicU64::new(0));
        let serve_root = Arc::clone(&root);
        let serve_gen = Arc::clone(&generation);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(stream) = stream {
                    let serve_root = Arc::clone(&serve_root);
                    let serve_gen = Arc::clone(&serve_gen);
                    thread::spawn(move || {
                        let _ = handle_client(stream, &serve_root, &serve_gen);
                    });
                }
            }
        });
        Ok(Self {
            port,
            root,
            generation,
        })
    }

    pub fn set_root(&self, root: PathBuf) {
        if let Ok(mut guard) = self.root.lock() {
            *guard = root;
        }
        self.bump_generation();
    }

    pub fn bump_generation(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    pub fn url_for(&self, slug: &str, pretty: bool) -> String {
        format!(
            "http://127.0.0.1:{}/{}",
            self.port,
            page_file_name(slug, pretty)
        )
    }
}

pub fn serve_dir_blocking(root: PathBuf, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let root = Arc::new(Mutex::new(root));
    let generation = Arc::new(AtomicU64::new(0));
    eprintln!(
        "Serving {} at http://127.0.0.1:{}/",
        root.lock().unwrap().display(),
        port
    );
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let root = Arc::clone(&root);
            let generation = Arc::clone(&generation);
            thread::spawn(move || {
                let _ = handle_client(stream, &root, &generation);
            });
        }
    }
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    root: &Arc<Mutex<PathBuf>>,
    generation: &Arc<AtomicU64>,
) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    if n == 0 {
        return Ok(());
    }
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = parse_path(&req).unwrap_or("/");
    let root_path = root
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|e| e.into_inner().clone());

    if path == RELOAD_PATH {
        let token = reload_token(&root_path, generation.load(Ordering::Relaxed));
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            token.len()
        );
        stream.write_all(header.as_bytes())?;
        stream.write_all(token.as_bytes())?;
        return Ok(());
    }

    let (status, mut body, ctype, extra_cache) = match resolve_file(&root_path, path) {
        Some((bytes, mime)) => ("200 OK", bytes, mime, html_cache_header(mime)),
        None => {
            let fallback = root_path.join("404.html");
            if let Ok(bytes) = std::fs::read(&fallback) {
                (
                    "404 Not Found",
                    bytes,
                    "text/html; charset=utf-8",
                    "Cache-Control: no-cache\r\n",
                )
            } else {
                (
                    "404 Not Found",
                    b"Not Found".to_vec(),
                    "text/plain; charset=utf-8",
                    "",
                )
            }
        }
    };
    if ctype.starts_with("text/html") {
        inject_reload_script(&mut body);
    }
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\n{extra_cache}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}

fn html_cache_header(mime: &str) -> &'static str {
    if mime.starts_with("text/html") {
        "Cache-Control: no-cache\r\n"
    } else {
        ""
    }
}

fn reload_token(root: &Path, generation: u64) -> String {
    let mtime = std::fs::metadata(root.join("index.html"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{generation}-{mtime}")
}

fn inject_reload_script(body: &mut Vec<u8>) {
    let Ok(html) = std::str::from_utf8(body) else {
        return;
    };
    if html.contains(RELOAD_PATH) {
        return;
    }
    if let Some(idx) = html.rfind("</body>") {
        let mut out = String::with_capacity(html.len() + RELOAD_SNIPPET.len());
        out.push_str(&html[..idx]);
        out.push_str(RELOAD_SNIPPET);
        out.push_str(&html[idx..]);
        *body = out.into_bytes();
    } else {
        body.extend_from_slice(RELOAD_SNIPPET.as_bytes());
    }
}

fn parse_path(req: &str) -> Option<&str> {
    let line = req.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    if method != "GET" && method != "HEAD" {
        return None;
    }
    let target = parts.next()?;
    Some(target.split('?').next().unwrap_or(target))
}

fn resolve_file(root: &Path, url_path: &str) -> Option<(Vec<u8>, &'static str)> {
    let trimmed = url_path.trim_start_matches('/');
    let rel = if trimmed.is_empty() {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(trimmed)
    };
    if rel
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return None;
    }
    let mut candidate = root.join(&rel);
    if candidate.is_dir() {
        candidate = candidate.join("index.html");
    }
    if !candidate.exists() {
        if let Some(name) = rel.file_name() {
            if !name.to_string_lossy().contains('.') {
                candidate = root.join(format!("{trimmed}.html"));
            }
        }
    }
    let bytes = std::fs::read(&candidate).ok()?;
    let mime = mime_for(&candidate);
    Some((bytes, mime))
}

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" | "webmanifest" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn serves_index_and_blocks_parent_dir() {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dd_serve_{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), b"<h1>hi</h1>").unwrap();
        let server = StaticServer::start(dir.clone()).expect("bind");
        let body = http_get(server.port, "/");
        assert!(body.contains("<h1>hi</h1>"), "got: {body}");
        assert!(
            body.contains("/__siteforge/reload"),
            "preview HTML should inject live-reload: {body}"
        );
        let denied = http_get(server.port, "/../Cargo.toml");
        assert!(denied.contains("Not Found") || denied.contains("404"));
        let reload = http_get(server.port, "/__siteforge/reload");
        assert!(reload.contains("\r\n\r\n"), "{reload}");
        let token = reload.split("\r\n\r\n").nth(1).unwrap_or("");
        assert!(token.contains('-'), "token={token}");
        server.bump_generation();
        let reload2 = http_get(server.port, "/__siteforge/reload");
        let token2 = reload2.split("\r\n\r\n").nth(1).unwrap_or("");
        assert_ne!(token, token2, "generation bump should change reload token");
        std::fs::remove_dir_all(&dir).ok();
    }

    fn http_get(port: u16, path: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    }
}
