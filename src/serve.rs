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
    if !request_host_allowed(&req) {
        let body = b"Bad Request";
        let header = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes())?;
        stream.write_all(body)?;
        return Ok(());
    }
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
            if let Some(bytes) = read_confined(&root_path, &fallback) {
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
    let decoded = percent_decode(url_path)?;
    let trimmed = decoded.trim_start_matches('/');
    let rel = if trimmed.is_empty() {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(trimmed)
    };
    if !rel_is_safe(&rel) {
        return None;
    }
    let mut candidate = root.join(&rel);
    if candidate.is_dir() {
        candidate = candidate.join("index.html");
    }
    if let Some(bytes) = read_confined(root, &candidate) {
        return Some((bytes, mime_for(&candidate)));
    }
    let name = rel.file_name()?.to_string_lossy();
    if name.contains('.') {
        return None;
    }
    let html_rel = PathBuf::from(format!("{trimmed}.html"));
    if !rel_is_safe(&html_rel) {
        return None;
    }
    let candidate = root.join(&html_rel);
    let bytes = read_confined(root, &candidate)?;
    Some((bytes, mime_for(&candidate)))
}

fn rel_is_safe(rel: &Path) -> bool {
    rel.components().all(|component| {
        matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    })
}

fn read_confined(root: &Path, file: &Path) -> Option<Vec<u8>> {
    let root = root.canonicalize().ok()?;
    let file = file.canonicalize().ok()?;
    if !file.starts_with(&root) || !file.is_file() {
        return None;
    }
    std::fs::read(file).ok()
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0 => return None,
            b'%' => {
                if i + 2 >= bytes.len() {
                    return None;
                }
                let hi = hex_val(bytes[i + 1])?;
                let lo = hex_val(bytes[i + 2])?;
                let decoded = (hi << 4) | lo;
                if decoded == 0 {
                    return None;
                }
                out.push(decoded);
                i += 3;
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn request_host_allowed(req: &str) -> bool {
    for line in req.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case("host") && !host_is_local(value.trim()) {
            return false;
        }
    }
    true
}

fn host_is_local(host: &str) -> bool {
    let host = host.trim();
    if host.is_empty() {
        return false;
    }
    if let Some(inner) = host.strip_prefix('[') {
        let Some(end) = inner.find(']') else {
            return false;
        };
        let after = &inner[end + 1..];
        if !(after.is_empty() || after.starts_with(':')) {
            return false;
        }
        return inner[..end].eq_ignore_ascii_case("::1");
    }
    let name = if let Some((name, _)) = host.rsplit_once(':') {
        if name.contains(':') {
            return false;
        }
        name
    } else {
        host
    };
    name.eq_ignore_ascii_case("localhost") || name == "127.0.0.1"
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

    #[test]
    fn preview_rejects_encoded_parent_bad_host_and_outside_symlink() {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("dd_serve_confine_{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), b"<h1>hi</h1>").unwrap();
        std::fs::write(dir.join("about"), b"about-page").unwrap();
        let parent_secret = dir
            .parent()
            .unwrap()
            .join(format!("dd_serve_parent_{n}.txt"));
        std::fs::write(&parent_secret, b"PARENTSECRET").unwrap();
        let outside = std::env::temp_dir().join(format!("dd_serve_outside_{n}"));
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), b"TOPSECRET").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(outside.join("secret.txt"), dir.join("leak.txt")).unwrap();
            std::os::unix::fs::symlink(outside.join("secret.txt"), dir.join("404.html")).unwrap();
        }
        let server = StaticServer::start(dir.clone()).expect("bind");

        let encoded_name = parent_secret.file_name().unwrap().to_string_lossy();
        let encoded = http_get(server.port, &format!("/%2e%2e/{encoded_name}"));
        assert!(
            !encoded.contains("PARENTSECRET"),
            "percent-decoded .. must not leave the export dir: {encoded}"
        );
        assert!(encoded.contains("404") || encoded.contains("Not Found"));
        let encoded_slash = http_get(server.port, &format!("/%2e%2e%2f{encoded_name}"));
        assert!(!encoded_slash.contains("PARENTSECRET"), "{encoded_slash}");

        let null_path = http_get(server.port, "/%00");
        assert!(null_path.contains("404") || null_path.contains("Not Found"));

        let extensionless = http_get(server.port, "/about");
        assert!(extensionless.contains("about-page"), "{extensionless}");

        let missing_host = http_get_host(server.port, "/", None);
        assert!(missing_host.contains("<h1>hi</h1>"), "{missing_host}");
        let local_host = http_get_host(server.port, "/", Some("localhost:9"));
        assert!(local_host.contains("<h1>hi</h1>"), "{local_host}");
        let v6 = http_get_host(server.port, "/", Some("[::1]:9"));
        assert!(v6.contains("<h1>hi</h1>"), "{v6}");

        let evil = http_get_host(server.port, "/", Some("evil.example"));
        assert!(evil.starts_with("HTTP/1.1 400"), "{evil}");
        assert!(!evil.contains("evil.example"), "{evil}");
        assert!(!evil.contains("<h1>hi</h1>"), "{evil}");

        #[cfg(unix)]
        {
            let leak = http_get(server.port, "/leak.txt");
            assert!(!leak.contains("TOPSECRET"), "{leak}");
            let missing = http_get(server.port, "/missing.html");
            assert!(!missing.contains("TOPSECRET"), "{missing}");
            assert!(missing.contains("Not Found"), "{missing}");
        }

        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&outside).ok();
        std::fs::remove_file(&parent_secret).ok();
    }

    fn http_get(port: u16, path: &str) -> String {
        http_get_host(port, path, Some("127.0.0.1"))
    }

    fn http_get_host(port: u16, path: &str, host: Option<&str>) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        let host_line = host
            .map(|value| format!("Host: {value}\r\n"))
            .unwrap_or_default();
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\n{host_line}Connection: close\r\n\r\n").as_bytes(),
            )
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    }
}
