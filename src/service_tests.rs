use super::Logger;
use crate::http::{serve, MAX_TOKEN_SIZE};
use crate::token::{clear_token, read_token, read_token_or_recover, write_token};
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::Server;

fn temporary_directory() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "mendeley-loopback-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn token_file_is_private_atomic_and_clearable() {
    let directory = temporary_directory();
    let path = directory.join("active-token.json");

    write_token(&path, "first-token").unwrap();
    assert_eq!(read_token(&path).as_deref(), Some("first-token"));
    write_token(&path, "second-token").unwrap();
    assert_eq!(read_token(&path).as_deref(), Some("second-token"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    clear_token(&path).unwrap();
    clear_token(&path).unwrap();
    assert_eq!(read_token(&path), None);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn token_reader_rejects_invalid_file_content() {
    let directory = temporary_directory();
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("active-token.json");

    fs::write(&path, r#"{"token":""}"#).unwrap();
    assert_eq!(read_token(&path), None);
    fs::write(&path, r#"{"token":7}"#).unwrap();
    assert_eq!(read_token(&path), None);
    fs::write(&path, "{").unwrap();
    assert_eq!(read_token(&path), None);

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn token_recovery_persists_only_when_no_saved_token_exists() {
    let directory = temporary_directory();
    let path = directory.join("active-token.json");
    let logger = Logger::new().unwrap();
    let recovered = read_token_or_recover(&path, || Some("recovered-token".into()), &logger);
    assert_eq!(recovered.as_deref(), Some("recovered-token"));
    assert_eq!(read_token(&path).as_deref(), Some("recovered-token"));
    let ignored = read_token_or_recover(&path, || panic!("saved token must win"), &logger);
    assert_eq!(ignored.as_deref(), Some("recovered-token"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn origin_policy_allows_only_local_web_origins_and_desktop_schemes() {
    for origin in [
        None,
        Some("null"),
        Some("file://"),
        Some("file:///plugin/index.html"),
        Some("app://onlyoffice"),
        Some("http://localhost:3000"),
        Some("https://127.0.0.1"),
        Some("http://[::1]:8080"),
    ] {
        assert!(
            crate::http::approved_origin(origin),
            "expected approved origin: {origin:?}"
        );
    }
    for origin in [
        Some("https://attacker.invalid"),
        Some("http://localhost.attacker.invalid"),
        Some("http://user@localhost"),
        Some("http://localhost/path"),
        Some("http://localhost:invalid"),
    ] {
        assert!(
            !crate::http::approved_origin(origin),
            "expected rejected origin: {origin:?}"
        );
    }
}

fn exchange(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    write!(stream, "{method} {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n").unwrap();
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").unwrap();
    }
    write!(
        stream,
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn oauth_routes_preserve_responses_and_token_lifecycle() {
    let directory = temporary_directory();
    let token_path = directory.join("active-token.json");
    let logger = Logger::new().unwrap();
    let server = Server::http("127.0.0.1:0").unwrap();
    let address = server.server_addr().to_ip().unwrap();
    let server_thread = thread::spawn(move || serve(server, &token_path, &logger).unwrap());

    let health = exchange(address, "GET", "/health", &[], b"");
    assert!(health.starts_with("HTTP/1.0 200"));
    assert!(health.contains("\"service\":\"mendeley-loopback\",\"status\":\"ok\""));

    for path in ["/", "/callback"] {
        let callback = exchange(address, "GET", path, &[], b"");
        assert!(callback.starts_with("HTTP/1.0 200"));
        assert!(callback.contains("fetch(\"/token\""));
    }

    let preflight = exchange(address, "OPTIONS", "/token", &[("Origin", "file://")], b"");
    assert!(preflight.starts_with("HTTP/1.0 204"));
    assert!(preflight.contains("Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS"));

    let saved = exchange(
        address,
        "POST",
        "/token",
        &[("Origin", "null"), ("Content-Type", "application/json")],
        br#"{"token":"oauth-token"}"#,
    );
    assert!(saved.starts_with("HTTP/1.0 200"));
    assert_eq!(
        read_token(&directory.join("active-token.json")).as_deref(),
        Some("oauth-token")
    );

    let retrieved = exchange(address, "GET", "/token", &[], b"");
    assert!(retrieved.contains(r#"{"token":"oauth-token"}"#));
    assert!(retrieved.contains("Cache-Control: no-store"));
    let invalid = exchange(
        address,
        "POST",
        "/token",
        &[("Origin", "null"), ("Content-Type", "application/json")],
        b"{",
    );
    assert!(invalid.starts_with("HTTP/1.0 400"));
    assert!(invalid.contains(r#"{"error":"invalid_token_payload"}"#));

    let oversized_token =
        serde_json::json!({"token": "x".repeat(crate::http::MAX_TOKEN_SIZE + 1)}).to_string();
    let oversized = exchange(
        address,
        "POST",
        "/token",
        &[("Origin", "null"), ("Content-Type", "application/json")],
        oversized_token.as_bytes(),
    );
    assert!(oversized.starts_with("HTTP/1.0 400"));
    assert!(oversized.contains(r#"{"error":"invalid_token_payload"}"#));

    let forbidden = exchange(
        address,
        "GET",
        "/token",
        &[("Origin", "https://attacker.invalid")],
        b"",
    );
    assert!(forbidden.starts_with("HTTP/1.0 403"));
    assert!(forbidden.contains(r#"{"error":"origin_not_allowed"}"#));

    let deleted = exchange(address, "DELETE", "/token", &[], b"");
    assert!(deleted.contains(r#"{"status":"cleared"}"#));
    assert_eq!(read_token(&directory.join("active-token.json")), None);
    let missing = exchange(address, "GET", "/missing", &[], b"");
    assert!(missing.starts_with("HTTP/1.0 404"));

    let shutdown = exchange(address, "POST", "/shutdown", &[], b"");
    assert!(shutdown.contains(r#"{"status":"stopping"}"#));
    server_thread.join().unwrap();
    fs::remove_dir_all(directory).unwrap();
}
