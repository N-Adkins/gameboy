use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tiny_http::{Header, Method, Response, Server, StatusCode};

fn main() {
    let project_root = std::env::current_dir().expect("cannot get cwd");
    let www_dir = project_root.join("www");

    // Build WASM with wasm-pack
    println!("Building WASM...");
    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web", "--out-dir", "www/pkg"])
        .status();

    match status {
        Ok(s) if s.success() => println!("WASM built successfully.\n"),
        Ok(_) => {
            eprintln!("wasm-pack build failed.");
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("wasm-pack not found. Install it with:");
            eprintln!("  cargo install wasm-pack");
            std::process::exit(1);
        }
    }

    let server = Server::http("127.0.0.1:8080").expect("failed to bind port 8080");
    println!("Serving http://localhost:8080");
    println!("Press Ctrl+C to stop.\n");

    webbrowser::open("http://localhost:8080").unwrap();

    for req in server.incoming_requests() {
        handle(req, &www_dir);
    }
}

fn handle(req: tiny_http::Request, www_dir: &Path) {
    if req.method() != &Method::Get {
        let _ = req.respond(Response::empty(StatusCode(405)));
        return;
    }

    let url = req.url().to_string();
    let url_path = url.split('?').next().unwrap_or("/");
    let url_path = if url_path == "/" {
        "/index.html"
    } else {
        url_path
    }
    .to_string();

    if url_path.contains("..") {
        let _ = req.respond(Response::empty(StatusCode(403)));
        return;
    }

    let rel = url_path.trim_start_matches('/');
    let file_path: PathBuf = www_dir.join(rel);

    match fs::read(&file_path) {
        Ok(body) => {
            let ct: Header = format!("Content-Type: {}", content_type(&url_path))
                .parse()
                .unwrap();
            let cors: Header = "Access-Control-Allow-Origin: *".parse().unwrap();
            let response = Response::from_data(body).with_header(ct).with_header(cors);
            println!("200 GET {url_path}");
            let _ = req.respond(response);
        }
        Err(_) => {
            let response = Response::from_string("404 Not Found").with_status_code(StatusCode(404));
            println!("404 GET {url_path}");
            let _ = req.respond(response);
        }
    }
}

fn content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}
