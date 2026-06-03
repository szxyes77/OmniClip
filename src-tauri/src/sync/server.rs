use std::sync::{Arc, Mutex};
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use sha2::{Digest, Sha256};
use crate::database::DatabaseManager;

const MDNS_PORT: u16 = 53531;
const DEFAULT_TCP_PORT: u16 = 18910;
const DEFAULT_HTTP_PORT: u16 = 18911;
const PROTOCOL_VERSION: u8 = 1;

static PAIR_CODE: AtomicU32 = AtomicU32::new(0);
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// SSE 事件广播器 - 允许多个 SSE 客户端订阅事件
pub struct EventBroadcaster {
    /// 所有 SSE 客户端的发送器 (id -> sender)
    clients: Mutex<HashMap<u64, crossbeam_channel::Sender<String>>>,
    next_id: AtomicU64,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
        }
    }

    /// 注册新的 SSE 客户端，返回 (client_id, receiver)
    pub fn subscribe(&self) -> (u64, crossbeam_channel::Receiver<String>) {
        let (tx, rx) = crossbeam_channel::bounded(32);
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.clients.lock().unwrap().insert(id, tx);
        println!("[SSE] Client #{} connected, total: {}", id, self.clients.lock().unwrap().len());
        (id, rx)
    }

    /// 取消注册 SSE 客户端
    pub fn unsubscribe(&self, id: u64) {
        self.clients.lock().unwrap().remove(&id);
        println!("[SSE] Client #{} disconnected, total: {}", id, self.clients.lock().unwrap().len());
    }

    /// 广播事件到所有 SSE 客户端
    pub fn broadcast(&self, event_type: &str, data: &str) {
        let event = format!("event: {}\ndata: {}\n\n", event_type, data);
        let mut clients = self.clients.lock().unwrap();
        let mut dead_ids = Vec::new();
        for (id, tx) in clients.iter() {
            // 非阻塞发送，如果客户端缓冲区满则标记为死亡
            if tx.try_send(event.clone()).is_err() {
                dead_ids.push(*id);
            }
        }
        // 清理失效的客户端
        for id in dead_ids {
            clients.remove(&id);
        }
        if !clients.is_empty() {
            println!("[SSE] Broadcast '{}' to {} clients", event_type, clients.len());
        }
    }
}

fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub struct SyncServer {
    _handles: Vec<thread::JoinHandle<()>>,
    tcp_port: u16,
    http_port: u16,
    broadcaster: Arc<EventBroadcaster>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
}

impl SyncServer {
    pub fn start(tcp_port: u16, db: Arc<Mutex<DatabaseManager>>) -> Self {
        let actual_tcp_port = if tcp_port > 0 { tcp_port } else { DEFAULT_TCP_PORT };
        let actual_http_port = DEFAULT_HTTP_PORT;

        generate_pair_code();

        let broadcaster = Arc::new(EventBroadcaster::new());

        let tcp_handle = Self::start_tcp_listener(actual_tcp_port, db.clone(), broadcaster.clone());
        let mdns_handle = Self::start_mdns_broadcast(actual_tcp_port);
        let mdns_discovery_handle = Self::start_mdns_discovery();
        let http_handle = Self::start_http_server(actual_http_port, db.clone(), broadcaster.clone());

        println!("[Sync] HTTP server running on http://0.0.0.0:{}", actual_http_port);
        println!("[Sync] TCP listener running on port {}", actual_tcp_port);
        println!("[Sync] Pair code: {}", get_pair_code());

        SyncServer {
            _handles: vec![tcp_handle, mdns_handle, mdns_discovery_handle, http_handle],
            tcp_port: actual_tcp_port,
            http_port: actual_http_port,
            broadcaster,
        }
    }

    pub fn get_pair_code() -> String {
        get_pair_code()
    }

    pub fn verify_pair_code(code: &str) -> bool {
        code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) && code != "000000"
    }

    fn start_tcp_listener(port: u16, db: Arc<Mutex<DatabaseManager>>, broadcaster: Arc<EventBroadcaster>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[Sync] Failed to start TCP listener on {}: {}", port, e);
                    return;
                }
            };
            println!("[Sync] TCP listener ready on {}", addr);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let db = db.clone();
                        let broadcaster = broadcaster.clone();
                        thread::spawn(move || {
                            if let Err(e) = Self::handle_tcp_connection(stream, &db, &broadcaster) {
                                eprintln!("[Sync] TCP error: {}", e);
                            }
                        });
                    }
                    Err(e) => eprintln!("[Sync] Accept error: {}", e),
                }
            }
        })
    }

    fn start_http_server(port: u16, db: Arc<Mutex<DatabaseManager>>, broadcaster: Arc<EventBroadcaster>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", port);
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[Sync] Failed to start HTTP server on {}: {}", port, e);
                    return;
                }
            };
            println!("[Sync] HTTP server ready on {}", addr);

            loop {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let db = db.clone();
                        let broadcaster = broadcaster.clone();
                        thread::spawn(move || {
                            if let Err(e) = Self::handle_http_request(stream, &db, &broadcaster) {
                                eprintln!("[Sync] HTTP error: {}", e);
                            }
                        });
                    }
                    Err(e) => eprintln!("[Sync] HTTP accept error: {}", e),
                }
            }
        })
    }

    fn start_mdns_broadcast(port: u16) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let hostname = get_hostname().unwrap_or_else(|| "omniclip-pc".to_string());
            let payload = format!("{}:{}:{}", PROTOCOL_VERSION, hostname, port);
            let msg = format!("{}{}", payload, "\0");

            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
                if socket.set_broadcast(true).is_ok() {
                    println!("[Sync] mDNS broadcasting: {}", payload);
                    loop {
                        let _ = socket.send_to(msg.as_bytes(), format!("255.255.255.255:{}", MDNS_PORT));
                        thread::sleep(Duration::from_secs(3));
                    }
                }
            }
        })
    }

    fn start_mdns_discovery() -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let addr = format!("0.0.0.0:{}", MDNS_PORT);
            let socket = match UdpSocket::bind(&addr) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Sync] mDNS discovery bind failed: {}", e);
                    return;
                }
            };

            println!("[Sync] mDNS discovery listening on {}", addr);

            let mut buf = [0u8; 1024];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        let msg = String::from_utf8_lossy(&buf[..len]);
                        if let Some(payload) = msg.strip_suffix('\0') {
                            let parts: Vec<&str> = payload.split(':').collect();
                            if parts.len() == 3 && parts[0] == "1" {
                                let _peer = PeerInfo {
                                    name: parts[1].to_string(),
                                    ip: src.ip().to_string(),
                                    port: parts[2].parse().unwrap_or(0),
                                };
                                println!("[Sync] Discovered: {} at {}:{}", _peer.name, _peer.ip, _peer.port);
                            }
                        }
                    }
                    Err(e) => eprintln!("[Sync] mDNS recv error: {}", e),
                }
            }
        })
    }

    fn handle_tcp_connection(mut stream: TcpStream, db: &Arc<Mutex<DatabaseManager>>, broadcaster: &EventBroadcaster) -> Result<(), String> {
        let peer = stream.peer_addr().map_err(|e| e.to_string())?.to_string();
        println!("[Sync-TCP] Connected from {}", peer);

        let _ = stream.write_all(b"OK:CONNECTED");
        stream.flush().ok();

        let mut buffer = Vec::with_capacity(65536);
        stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
        let mut temp = [0u8; 4096];

        loop {
            buffer.clear();
            match stream.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&temp[..n]);
                    if buffer.len() > 60000 { break; }
                }
                Err(_) => break,
            }

            let message = String::from_utf8_lossy(&buffer).to_string();
            let message = message.trim();

            if message.starts_with("SYNC:TEXT:") {
                let text = &message[10..];
                if !text.is_empty() {
                    Self::store_to_clipboard(text, db, broadcaster);
                    println!("[Sync-TCP] Stored {} chars from {}", text.len(), peer);
                    let _ = stream.write_all(b"OK:TEXT_RECEIVED\n");
                    stream.flush().ok();
                }
            } else if message == "SYNC:REQUEST:RECORDS" {
                let db_lock = db.lock().unwrap();
                match db_lock.get_records(50, 0) {
                    Ok(records) => {
                        let json = serde_json::to_string(&records).unwrap_or_default();
                        let response = format!("DATA:{}", json);
                        let _ = stream.write_all(response.as_bytes());
                        stream.flush().ok();
                        println!("[Sync-TCP] Sent {} records to {}", records.len(), peer);
                    }
                    Err(e) => {
                        let _ = stream.write_all(format!("ERR:{}", e).as_bytes());
                        stream.flush().ok();
                    }
                }
            } else if message == "PING" {
                let _ = stream.write_all(b"PONG\n");
                stream.flush().ok();
            }
        }

        println!("[Sync-TCP] Disconnected {}", peer);
        Ok(())
    }

    fn store_to_clipboard(text: &str, db: &Arc<Mutex<DatabaseManager>>, broadcaster: &EventBroadcaster) {
        // Set system clipboard
        use arboard::Clipboard;
        if let Ok(mut c) = Clipboard::new() {
            let _ = c.set_text(text);
            println!("[Sync] Set system clipboard ({} chars)", text.len());
        }

        // Store to database
        let db_lock = db.lock().unwrap();
        let content_hash = compute_content_hash(text);
        let title = if text.len() > 50 { &text[..50] } else { text };
        if let Ok(record) = db_lock.insert_record("text", &content_hash, text, title, None) {
            drop(db_lock);
            // Broadcast SSE event to all mobile clients
            let json = serde_json::to_string(&record).unwrap_or_default();
            broadcaster.broadcast("clipboard-update", &json);
            println!("[Sync] Stored record, broadcast to SSE clients");
        } else {
            drop(db_lock);
            println!("[Sync] Record already exists (duplicate content), skipping broadcast");
        }
    }

    fn handle_http_request(mut stream: TcpStream, db: &Arc<Mutex<DatabaseManager>>, broadcaster: &EventBroadcaster) -> Result<(), String> {
        let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).map_err(|e| e.to_string())?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 { return Ok(()); }

        let method = parts[0].to_string();
        let path_with_query = parts[1].to_string();
        let path = path_with_query.split('?').next().unwrap_or("").to_string();

        // Read headers
        let mut content_length: usize = 0;
        let mut body = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(|e| e.to_string())?;
            let line = line.trim();
            if line.is_empty() { break; }
            if line.to_lowercase().starts_with("content-length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }
            }
        }
        if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            reader.read_exact(&mut buf).ok();
            body = String::from_utf8_lossy(&buf).to_string();
        }

        if path == "/" || path == "/index.html" {
            let html = Self::render_web_ui();
            Self::send_response(&mut stream, 200, "text/html; charset=utf-8", &html);
        } else if path == "/api/pair-code" && method == "GET" {
            Self::send_response(&mut stream, 200, "application/json",
                &format!(r#"{{"code":"{}","port":{}}}"#, get_pair_code(), DEFAULT_HTTP_PORT));
        } else if path == "/api/verify" && method == "POST" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(code) = json.get("code").and_then(|v| v.as_str()) {
                    if Self::verify_pair_code(code) {
                        Self::send_response(&mut stream, 200, "application/json", r#"{"success":true,"message":"Paired successfully"}"#);
                    } else {
                        Self::send_response(&mut stream, 400, "application/json", r#"{"success":false,"message":"Invalid pair code"}"#);
                    }
                } else {
                    Self::send_response(&mut stream, 400, "application/json", r#"{"success":false,"message":"no code"}"#);
                }
            } else {
                Self::send_response(&mut stream, 400, "application/json", r#"{"success":false,"message":"invalid json"}"#);
            }
        } else if path == "/api/events" && method == "GET" {
            // SSE endpoint - mobile connects here for real-time updates
            Self::handle_sse_connection(&mut stream, broadcaster);
            return Ok(());
        } else if path == "/api/records" && method == "GET" {
            let db_lock = db.lock().unwrap();
            match db_lock.get_records(100, 0) {
                Ok(records) => {
                    let json = serde_json::to_string(&records).unwrap_or_default();
                    Self::send_response(&mut stream, 200, "application/json", &json);
                }
                Err(e) => {
                    Self::send_response(&mut stream, 500, "application/json", &format!(r#"{{"error":"{}"}}"#, e));
                }
            }
        } else if path == "/api/search" && method == "GET" {
            let query_str = path_with_query.split('?').nth(1).unwrap_or("");
            let query = query_str.strip_prefix("q=").unwrap_or("").replace("%20", " ");
            let db_lock = db.lock().unwrap();
            match db_lock.search_records(&query, 50) {
                Ok(records) => {
                    let json = serde_json::to_string(&records).unwrap_or_default();
                    Self::send_response(&mut stream, 200, "application/json", &json);
                }
                Err(e) => {
                    Self::send_response(&mut stream, 500, "application/json", &format!(r#"{{"error":"{}"}}"#, e));
                }
            }
        } else if path == "/api/ip" && method == "GET" {
            let ip = get_local_ip();
            Self::send_response(&mut stream, 200, "application/json", &format!(r#"{{"ip":"{}"}}"#, ip));
        } else if path == "/api/sync" && method == "POST" {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                    Self::store_to_clipboard(text, db, broadcaster);
                    println!("[Sync-HTTP] Stored {} chars from web", text.len());
                    Self::send_response(&mut stream, 200, "application/json", r#"{"status":"ok"}"#);
                } else {
                    Self::send_response(&mut stream, 400, "application/json", r#"{"error":"no text field"}"#);
                }
            } else {
                Self::send_response(&mut stream, 400, "application/json", r#"{"error":"invalid json"}"#);
            }
        } else if path.starts_with("/api/star/") && method == "POST" {
            let id = path.trim_start_matches("/api/star/");
            let db_lock = db.lock().unwrap();
            match db_lock.toggle_star(id) {
                Ok(starred) => {
                    Self::send_response(&mut stream, 200, "application/json", &format!(r#"{{"status":"ok","starred":{}}}"#, starred));
                }
                Err(e) => {
                    Self::send_response(&mut stream, 500, "application/json", &format!(r#"{{"error":"{}"}}"#, e));
                }
            }
        } else if path.starts_with("/api/delete/") && (method == "POST" || method == "DELETE") {
            let id = path.trim_start_matches("/api/delete/");
            let db_lock = db.lock().unwrap();
            match db_lock.delete_record(id) {
                Ok(_) => {
                    Self::send_response(&mut stream, 200, "application/json", r#"{"status":"ok"}"#);
                }
                Err(e) => {
                    Self::send_response(&mut stream, 500, "application/json", &format!(r#"{{"error":"{}"}}"#, e));
                }
            }
        } else if path.starts_with("/api/paste/") && (method == "POST" || method == "PUT") {
            let id = path.trim_start_matches("/api/paste/");
            let content = {
                let db_lock = db.lock().unwrap();
                match db_lock.get_record_by_id(id) {
                    Ok(record) => record.content.clone(),
                    Err(_) => {
                        Self::send_response(&mut stream, 404, "application/json", r#"{"error":"not found"}"#);
                        return Ok(());
                    }
                }
            };
            Self::store_to_clipboard(&content, db, broadcaster);
            let _ = db.lock().unwrap().increment_copy_count(id);
            Self::send_response(&mut stream, 200, "application/json", r#"{"status":"ok"}"#);
        } else {
            Self::send_response(&mut stream, 404, "text/plain", "Not Found");
        }

        Ok(())
    }

    fn handle_sse_connection(stream: &mut TcpStream, broadcaster: &EventBroadcaster) {
        let (client_id, rx) = broadcaster.subscribe();

        // Send SSE headers
        let headers = "HTTP/1.1 200 OK\r\n\
            Content-Type: text/event-stream\r\n\
            Cache-Control: no-cache\r\n\
            Connection: keep-alive\r\n\
            Access-Control-Allow-Origin: *\r\n\
            X-Accel-Buffering: no\r\n\r\n";
        if stream.write_all(headers.as_bytes()).is_err() {
            broadcaster.unsubscribe(client_id);
            return;
        }
        if stream.flush().is_err() {
            broadcaster.unsubscribe(client_id);
            return;
        }

        println!("[SSE] Connection established for client #{}", client_id);

        // Send initial connection event
        let welcome = format!("event: connected\ndata: {{\"id\":{}}}\n\n", client_id);
        if stream.write_all(welcome.as_bytes()).is_err() {
            broadcaster.unsubscribe(client_id);
            return;
        }
        let _ = stream.flush();

        // Keep connection alive with periodic comments and forward events
        let mut last_activity = std::time::Instant::now();
        let timeout = Duration::from_secs(300); // 5 minute timeout

        loop {
            // Try to receive an event (non-blocking with timeout)
            match rx.recv_timeout(Duration::from_secs(15)) {
                Ok(event) => {
                    if stream.write_all(event.as_bytes()).is_err() {
                        break;
                    }
                    if stream.flush().is_err() {
                        break;
                    }
                    last_activity = std::time::Instant::now();
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Send keep-alive comment
                    if stream.write_all(b": keepalive\n\n").is_err() {
                        break;
                    }
                    if stream.flush().is_err() {
                        break;
                    }
                    // Check if connection has been idle too long
                    if last_activity.elapsed() > timeout {
                        println!("[SSE] Client #{} timed out", client_id);
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        broadcaster.unsubscribe(client_id);
        println!("[SSE] Connection closed for client #{}", client_id);
    }

    fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &str) {
        let status_text = match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, DELETE, PUT, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n{}",
            status, status_text, content_type, body.len(), body
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }

    fn render_web_ui() -> String {
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>OmniClip Sync</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#f5f5f5;color:#333;max-width:600px;margin:0 auto;padding:16px}
h1{color:#4361ee;margin-bottom:16px}
.pair-code{font-size:48px;font-weight:700;letter-spacing:8px;color:#4361ee;background:#fff;padding:20px;border-radius:12px;text-align:center;margin:16px 0}
textarea,input{width:100%;padding:12px;border:1px solid #d0d0d0;border-radius:8px;font-size:14px;margin:8px 0}
button{padding:12px 24px;background:#4361ee;color:#fff;border:none;border-radius:8px;cursor:pointer;margin:4px}
.record{background:#fff;padding:12px;margin:8px 0;border-radius:8px;border:1px solid #e8e8e8}
.record-actions{margin-top:8px}
.record-actions button{padding:6px 12px;font-size:12px}
</style>
</head>
<body>
<h1>OmniClip 同步</h1>
<div class="pair-code" id="pairCode">------</div>
<h3>发送文本</h3>
<textarea id="sendText" placeholder="输入文本"></textarea>
<button onclick="sendText()">发送到剪贴板</button>
<h3>剪贴板记录</h3>
<div id="records"></div>
<script>
const API = location.origin;
async function loadRecords() {
  try {
    const r = await fetch(API + '/api/records');
    const data = await r.json();
    document.getElementById('records').innerHTML = data.map(r =>
      '<div class="record"><div>' + r.content.substring(0, 100) + '</div><div class="record-actions"><button onclick="paste(\'' + r.id + '\')">复制到剪贴板</button></div></div>'
    ).join('');
  } catch(e) { console.error(e); }
}
async function sendText() {
  const text = document.getElementById('sendText').value;
  if (!text) return;
  try {
    await fetch(API + '/api/sync', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({text}) });
    alert('已发送!');
    document.getElementById('sendText').value = '';
    loadRecords();
  } catch(e) { alert('发送失败: ' + e.message); }
}
async function paste(id) {
  try {
    await fetch(API + '/api/paste/' + id, {method: 'POST'});
    alert('已复制!');
  } catch(e) { alert('失败: ' + e.message); }
}
async function loadPairCode() {
  try {
    const r = await fetch(API + '/api/pair-code');
    const data = await r.json();
    document.getElementById('pairCode').textContent = data.code;
  } catch(e) {}
}
loadRecords();
loadPairCode();
setInterval(loadRecords, 3000);
</script>
</body>
</html>"#
        .to_string()
    }

    /// Send text to a peer device (mobile)
    /// Stores to database and triggers SSE event for mobile to receive via SSE connection
    pub fn send_text_to_peer(peer_ip: &str, peer_port: u16, text: &str, db: &Arc<Mutex<DatabaseManager>>, broadcaster: &EventBroadcaster) -> Result<(), String> {
        println!("[Sync] Sending text to peer {}:{}", peer_ip, peer_port);

        // First try to send directly via TCP if mobile supports it (port 18910)
        if peer_port == 18910 || peer_port == DEFAULT_TCP_PORT {
            let addr = format!("{}:{}", peer_ip, peer_port);
            if let Ok(mut stream) = TcpStream::connect(&addr) {
                stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                let msg = format!("SYNC:TEXT:{}", text);
                if stream.write_all(msg.as_bytes()).is_ok() {
                    stream.flush().ok();
                    // Read response
                    let mut response = String::new();
                    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    let _ = stream.read_to_string(&mut response);
                    println!("[Sync] TCP send to {}:{} succeeded: {}", peer_ip, peer_port, response.trim());
                    // Also store to our database for history
                    Self::store_to_clipboard(text, db, broadcaster);
                    return Ok(());
                }
            }
            println!("[Sync] TCP send to {}:{} failed, falling back to HTTP+SSE", peer_ip, peer_port);
        }

        // Fallback: Try HTTP POST to mobile's server (if mobile runs one on peer_port)
        if peer_port != DEFAULT_HTTP_PORT && peer_port != DEFAULT_TCP_PORT {
            let addr = format!("{}:{}", peer_ip, peer_port);
            if let Ok(mut stream) = TcpStream::connect(&addr) {
                stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                let json_body = serde_json::json!({"text": text}).to_string();
                let request = format!(
                    "POST /api/sync HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    addr, json_body.len(), json_body
                );
                if stream.write_all(request.as_bytes()).is_ok() {
                    stream.flush().ok();
                    let mut response = String::new();
                    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    let _ = stream.read_to_string(&mut response);
                    if response.contains("200") || response.contains("ok") {
                        println!("[Sync] HTTP send to {}:{} succeeded", peer_ip, peer_port);
                        Self::store_to_clipboard(text, db, broadcaster);
                        return Ok(());
                    }
                }
                println!("[Sync] HTTP send to {}:{} failed", peer_ip, peer_port);
            }
        }

        // Final fallback: Store locally and broadcast via SSE
        // Mobile will receive it via its SSE connection to PC
        Self::store_to_clipboard(text, db, broadcaster);
        println!("[Sync] Stored text locally, mobile will receive via SSE");
        Ok(())
    }

    pub fn request_records_from_peer(peer_ip: &str, peer_port: u16) -> Result<String, String> {
        let addr = format!("{}:{}", peer_ip, peer_port);
        let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
        stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

        stream.write_all(b"SYNC:REQUEST:RECORDS").map_err(|e| e.to_string())?;

        let mut response = String::new();
        stream.read_to_string(&mut response).map_err(|e| e.to_string())?;

        if response.starts_with("DATA:") {
            Ok(response[5..].to_string())
        } else {
            Err("Peer returned invalid data".to_string())
        }
    }

    /// Get the broadcaster reference for external use
    pub fn get_broadcaster(&self) -> Arc<EventBroadcaster> {
        self.broadcaster.clone()
    }
}

fn get_hostname() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
}

fn get_local_ip() -> String {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn generate_pair_code() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let code = ((seed * 1234567 + 7654321) % 900000 + 100000) as u32;
    PAIR_CODE.store(code, Ordering::SeqCst);
}

fn get_pair_code() -> String {
    format!("{:06}", PAIR_CODE.load(Ordering::SeqCst))
}
