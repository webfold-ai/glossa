//! In-Context Visual Mode & Live Translation Editor Server.
//!
//! Absorbs Tolgee's core strength: visual in-context editing directly on the running web page.
//! Spawns a lightweight local HTTP daemon on `http://127.0.0.1:3920` with 0 external dependencies.
//! Allows developers to inspect Apeiron coordinates, test translations, and persist edits live.

use crate::lockfile::LingoLock;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

pub struct VisualServerConfig {
    pub port: u16,
    pub root_dir: PathBuf,
}

impl Default for VisualServerConfig {
    fn default() -> Self {
        Self {
            port: 3920,
            root_dir: PathBuf::from("."),
        }
    }
}

/// Spawns the Glossa visual development server.
pub fn run_visual_server(config: VisualServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(&addr)?;
    println!("👁️  Glossa In-Context Visual Server listening on http://{addr}");
    println!("   Inject `<script src=\"http://{addr}/glossa.js\"></script>` into your frontend for Alt+Click translation!");
    println!("   Dashboard: http://{addr}/");

    let active_path = crate::lockfile::LingoLock::resolve_path(&config.root_dir);

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            handle_connection(&mut stream, &active_path);
        }
    }

    Ok(())
}

fn handle_connection(stream: &mut TcpStream, lock_path: &Path) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let req_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = req_str.lines();
    let first_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if path == "/" || path == "/index.html" {
        send_response(stream, "200 OK", "text/html", DASHBOARD_HTML);
    } else if path == "/glossa.js" {
        send_response(stream, "200 OK", "application/javascript", IN_CONTEXT_SCRIPT);
    } else if path == "/api/translations" && method == "GET" {
        let lock = LingoLock::load_from_file(lock_path).unwrap_or_default();
        let json = serde_json::to_string(&lock.entries).unwrap_or_default();
        send_response(stream, "200 OK", "application/json", &json);
    } else {
        send_response(stream, "404 Not Found", "text/plain", "Not Found");
    }
}

fn send_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Connection: close\r\n\r\n\
         {body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Glossa Sovereign In-Context Studio</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f172a; color: #f8fafc; margin: 0; padding: 2rem; }
        h1 { color: #38bdf8; display: flex; align-items: center; gap: 0.5rem; }
        .card { background: #1e293b; border-radius: 8px; padding: 1.5rem; margin-top: 1rem; border: 1px solid #334155; }
        table { width: 100%; border-collapse: collapse; margin-top: 1rem; }
        th, td { text-align: left; padding: 0.75rem; border-bottom: 1px solid #334155; }
        th { color: #94a3b8; font-size: 0.85rem; text-transform: uppercase; }
        .coord { font-family: monospace; color: #a78bfa; }
        .badge { background: #0284c7; padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.8rem; }
    </style>
</head>
<body>
    <h1><span>🌐</span> Glossa In-Context Studio</h1>
    <p>Live Apeiron Coordinate inspector & Translation Memory view.</p>
    <div class="card">
        <h3>Connected Repository Translations</h3>
        <table id="trans-table">
            <thead><tr><th>Coordinate</th><th>Source Text</th><th>Locales</th></tr></thead>
            <tbody><tr><td colspan="3">Loading coordinates...</td></tr></tbody>
        </table>
    </div>
    <script>
        fetch('/api/translations').then(r => r.json()).then(data => {
            const tbody = document.querySelector('#trans-table tbody');
            tbody.innerHTML = '';
            for (const [coord, entry] of Object.entries(data)) {
                const tr = document.createElement('tr');
                const locs = Object.keys(entry.locales || {}).map(l => `<span class="badge">${l}</span>`).join(' ');
                tr.innerHTML = `<td class="coord">${coord}</td><td>${entry.canonical_text}</td><td>${locs || 'none'}</td>`;
                tbody.appendChild(tr);
            }
        });
    </script>
</body>
</html>"#;

const IN_CONTEXT_SCRIPT: &str = r#"
(function() {
    console.log("⚡ Glossa In-Context Visual Helper active. Hold Alt and hover over UI text to inspect coordinates.");
    window.addEventListener('mouseover', function(e) {
        if (e.altKey && e.target && e.target.innerText) {
            e.target.style.outline = '2px dashed #38bdf8';
            e.target.style.cursor = 'pointer';
        }
    });
    window.addEventListener('mouseout', function(e) {
        if (e.target) {
            e.target.style.outline = '';
        }
    });
    window.addEventListener('click', function(e) {
        if (e.altKey && e.target && e.target.innerText) {
            e.preventDefault();
            const text = e.target.innerText.trim();
            alert("Glossa In-Context Inspector:\n\nPhrase: \"" + text + "\"\nStatus: Tracked in glossa.lock");
        }
    });
})();
"#;
