use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io;
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

static NOTE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]{1,64}$").unwrap());
static RANDOM_ALPHABET: &[u8] = b"234579abcdefghjkmnpqrstwxyz"; // 与 PHP 版本一致

#[derive(Clone)]
struct AppState {
    save_path: Arc<PathBuf>,
    file_limit: usize,
    single_file_size_limit: usize,
    static_root: Arc<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let save_path = env::var("SAVE_PATH").unwrap_or_else(|_| "_tmp".to_string());
    let file_limit = env::var("FILE_LIMIT").ok().and_then(|s| s.parse().ok()).unwrap_or(100000);
    let single_file_size_limit = env::var("SINGLE_FILE_SIZE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10240);
    let static_root = env::var("STATIC_ROOT").unwrap_or_else(|_| ".".to_string());

    fs::create_dir_all(&save_path)?;

    let state = AppState {
        save_path: Arc::new(PathBuf::from(save_path)),
        file_limit,
        single_file_size_limit,
        static_root: Arc::new(PathBuf::from(static_root)),
    };

    let app = Router::new()
        .route("/", get(get_root))
        .route("/:note", get(get_note).post(post_note))
        .route("/upload", post(upload_file))
        .route("/_tmp/:file", get(serve_tmp_file))
        // 静态资源（映射到现有文件）
        .route("/styles.css", get(serve_file))
        .route("/clippy.svg", get(serve_file))
        .route("/favicon.ico", get(serve_file))
        .route("/script.js", get(serve_file))
        .route("/copy.js", get(serve_file))
        .route("/markdown.js", get(serve_file))
        .route("/history.js", get(serve_file))
        .route("/js/:file", get(serve_public_js))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    info!("listening on {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn get_root() -> impl IntoResponse {
    Redirect::to(&format!("/{}", random_note_id(5)))
}

#[derive(Deserialize, Default)]
struct NoteQuery {
    raw: Option<String>,
}

async fn get_note(
    State(state): State<AppState>,
    Path(note): Path<String>,
    Query(query): Query<NoteQuery>,
    headers: HeaderMap,
) -> Response {
    // 校验 note
    if !NOTE_RE.is_match(&note) {
        return Redirect::to(&format!("/{}", random_note_id(5))).into_response();
    }

    let note_path = state.save_path.join(&note);

    // no-cache 头
    let base_headers = no_cache_headers();

    // raw 输出或 curl/wget UA
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_cli = ua.starts_with("curl") || ua.starts_with("Wget");
    let want_raw = query.raw.is_some() || is_cli;

    if want_raw {
        if note_path.is_file() {
            let Ok(bytes) = fs::read(&note_path) else {
                return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
            };
            let mut resp = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain; charset=utf-8")
                .body(bytes.into())
                .unwrap();
            resp.headers_mut().extend(base_headers.clone());
            return resp;
        } else {
            let mut resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap();
            resp.headers_mut().extend(base_headers.clone());
            return resp;
        }
    }

    // HTML 页面
    let content_escaped = if note_path.is_file() {
        match fs::read_to_string(&note_path) {
            Ok(s) => html_escape(&s),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    let excerpt = generate_excerpt_by_path(&note_path);
    let html = render_html(&note, &content_escaped, &excerpt);
    let mut resp = Html(html).into_response();
    resp.headers_mut().extend(base_headers);
    resp
}

#[derive(Deserialize)]
struct PostForm {
    text: Option<String>,
}

async fn post_note(
    State(state): State<AppState>,
    Path(note): Path<String>,
    Form(form): Form<PostForm>,
) -> Response {
    if !NOTE_RE.is_match(&note) {
        return Redirect::to(&format!("/{}", random_note_id(5))).into_response();
    }

    let text = form.text.unwrap_or_default();

    // 文件数量限制
    match count_files_in_dir(&state.save_path) {
        Ok(count) if count >= state.file_limit => {
            error!("File limit reached {}", state.file_limit);
            return StatusCode::FORBIDDEN.into_response();
        }
        Ok(_) => {}
        Err(e) => {
            error!("count files error: {e}");
        }
    }

    // 单文件大小限制（按字节计算）
    if text.as_bytes().len() > state.single_file_size_limit {
        error!("File size limit reached {}", state.single_file_size_limit);
        return StatusCode::FORBIDDEN.into_response();
    }

    let note_path = state.save_path.join(&note);
    if text.is_empty() {
        // 删除文件（如果存在）
        if note_path.exists() {
            let _ = fs::remove_file(&note_path);
        }
    } else {
        if let Err(e) = fs::write(&note_path, text) {
            error!("write error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    StatusCode::OK.into_response()
}

async fn serve_file(State(state): State<AppState>, uri: Uri) -> impl IntoResponse {
    // 从 static_root 读取同名文件
    let rel = uri.path().trim_start_matches('/');
    let path = state.static_root.join(rel);
    match fs::read(&path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = no_cache_headers();
            headers.insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());
            let mut resp = Response::builder().status(StatusCode::OK).body(bytes.into()).unwrap();
            resp.headers_mut().extend(headers);
            resp
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_public_js(State(state): State<AppState>, Path(file): Path<String>) -> impl IntoResponse {
    let safe = file.replace("../", "");
    let path = state.static_root.join("public").join("js").join(safe);
    match fs::read(&path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = no_cache_headers();
            headers.insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());
            let mut resp = Response::builder().status(StatusCode::OK).body(bytes.into()).unwrap();
            resp.headers_mut().extend(headers);
            resp
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_tmp_file(State(state): State<AppState>, Path(file): Path<String>) -> impl IntoResponse {
    let safe = file.replace("../", "");
    let path = state.save_path.join(safe);
    match fs::read(&path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mut headers = no_cache_headers();
            headers.insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());
            let mut resp = Response::builder().status(StatusCode::OK).body(bytes.into()).unwrap();
            resp.headers_mut().extend(headers);
            resp
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn upload_file(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    // 限制 100MB
    const MAX_SIZE: usize = 100 * 1024 * 1024;

    // 保存到 _tmp 下，文件名加时间戳避免冲突
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.name().map(|s| s.to_string()) {
            if name != "file" { continue; }
        }

        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| "upload.bin".to_string());
        let data = match field.bytes().await {
            Ok(b) => b,
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid file").into_response(),
        };
        if data.len() > MAX_SIZE { return (StatusCode::FORBIDDEN, "file too large").into_response(); }

        let ext = std::path::Path::new(&file_name).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let ts = chrono_like_timestamp();
        let safe_name = sanitize_filename(&file_name);
        let stored = format!("{ts}_{safe_name}");
        let path = state.save_path.join(&stored);

        if let Err(e) = fs::write(&path, &data) {
            error!("upload write error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        // 返回相对路径供前端插入 `_tmp/<name>`
        let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg");
        let url = format!("/_tmp/{}", stored);
        let json = serde_json::json!({
            "url": url,
            "is_image": is_image,
            "name": stored,
        });
        let resp = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(serde_json::to_vec(&json).unwrap().into())
            .unwrap();
        return resp;
    }

    (StatusCode::BAD_REQUEST, "no file").into_response()
}

fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}

fn sanitize_filename(name: &str) -> String {
    let mut s = name.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_");
    if s.is_empty() { s = "file".to_string(); }
    s
}

fn no_cache_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(
        "Cache-Control",
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    h.insert("Pragma", HeaderValue::from_static("no-cache"));
    h.insert("Expires", HeaderValue::from_static("0"));
    h
}

fn count_files_in_dir(dir: &FsPath) -> io::Result<usize> {
    let mut count = 0usize;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() {
            count += 1;
        }
    }
    Ok(count)
}

fn generate_excerpt_by_path(path: &FsPath) -> String {
    if path.is_file() {
        if let Ok(s) = fs::read_to_string(path) {
            return generate_excerpt(&s, 150);
        }
    }
    String::new()
}

fn generate_excerpt(text: &str, length: usize) -> String {
    let mut s = text.chars().take(length).collect::<String>();
    if text.chars().count() > length {
        s.push_str("...");
    }
    s
}

fn random_note_id(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..RANDOM_ALPHABET.len());
            RANDOM_ALPHABET[idx] as char
        })
        .collect()
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
}

fn render_html(note: &str, content_escaped: &str, excerpt: &str) -> String {
    // 前半部分用 format! 插入变量
    let mut html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>web-mini-note · {note}</title>
    <link rel="shortcut icon" href="/favicon.ico">
    <link rel="stylesheet" href="/styles.css">
    <meta name="description" content="📔 {desc}">
    <script src="/js/qrcode.min.js"></script>
    <script src="/js/clipboard.min.js"></script>
    <script src="/js/marked.min.js"></script>
    <script src="/js/mousetrap.min.js"></script>
</head>
<body>
    <div id="sidebar" class="sidebar">
        <script src="/history.js"></script>
        <span class="close-btn" onclick="toggleSidebar()">&times;</span>
        <h3>Recent Notes</h3>
        <ul id="history-list"></ul>
    </div>
    <div class="container">
        <div id="qrcodePopup">
            <div id="qrcode"></div>
        </div>
        <textarea class="mousetrap" id="content" spellcheck="false" autocapitalize="off" autocomplete="off" autocorrect="off">{content}</textarea>
        <button id="clippy" class="btn">
            <img src="/clippy.svg" alt="Copy to clipboard" style="width: 12px; height: 16px;">
        </button>
        <div id="markdown-content" style="display: none"></div>
        <div class="link">
            <a href="/">💡 new &nbsp;|&nbsp;</a>
            <a href="#" id="renderMarkdown">note/{note}&nbsp;<label id="renderStatus" style="cursor: pointer">🔓</label></a>
            <a href="#" id="showQRCode" class="copyBtn">&nbsp; | &nbsp;🔗 share</a>
            <a href="#" id="showHistory" class="showHistory">&nbsp; | &nbsp;📜 history</a>
            <a href="#" id="uploadTrigger">&nbsp; | &nbsp;⤴ upload</a>
        </div>
    </div>
    <pre id="printable"></pre>
    <div id="qrcode"></div>
    <script src="/markdown.js"></script>
    <script src="/copy.js"></script>
    <script src="/script.js"></script>
    <input type="file" id="fileInput" style="display:none" />
"##,
        note = note,
        content = content_escaped,
        desc = html_attr_escape(&format!("{}", excerpt)),
    );

    // 纯 JS 片段用原始字符串拼接，避免 format! 解析花括号
    const UPLOAD_JS: &str = r##"
    <script>
    (function(){
      var el = document.getElementById('uploadTrigger');
      var input = document.getElementById('fileInput');
      var ta = document.getElementById('content');
      if(el && input && ta){
        el.addEventListener('click', function(e){ e.preventDefault(); input.click(); });
        input.addEventListener('change', async function(){
          if(!input.files || input.files.length === 0) return;
          var f = input.files[0];
          if(f.size > 100*1024*1024){ showNotification('file too large (>100MB)'); return; }
          var fd = new FormData();
          fd.append('file', f);
          try{
            showNotification('uploading...');
            var resp = await fetch('/upload', { method: 'POST', body: fd });
            if(!resp.ok){ showNotification('upload failed'); return; }
            var data = await resp.json();
            var cursorPos = ta.selectionStart || 0;
            var before = ta.value.substring(0, cursorPos);
            var after = ta.value.substring(cursorPos);
            var insert = '';
            if(data.is_image){
              insert = '![](' + data.url + ')';
            } else {
              insert = '[' + (data.name || 'attachment') + '](' + data.url + ')';
            }
            ta.value = before + insert + after;
            ta.selectionStart = ta.selectionEnd = cursorPos + insert.length;
            ta.focus();
            showNotification('uploaded');
          }catch(e){
            showNotification('upload error');
          }finally{
            input.value = '';
          }
        });
      }
    })();
    </script>
    </body>
    </html>
    "##;

    html.push_str(UPLOAD_JS);
    html
}

fn html_attr_escape(input: &str) -> String {
    // 对于 meta content
    html_escape(&input.replace('"', "\""))
}

