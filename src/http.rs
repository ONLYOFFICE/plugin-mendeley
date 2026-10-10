use crate::logger::Logger;
use crate::service::{PORT, SERVICE_NAME};
use crate::token::{clear_token, read_token, write_token};
use serde_json::{json, Value};
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub(crate) const MAX_BODY_SIZE: usize = 16_384;
pub(crate) const MAX_TOKEN_SIZE: usize = 8_192;
const HTML_CALLBACK: &str = r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Mendeley Login Successful</title></head>
<body><h2>Sign-In Successful</h2><p>Mendeley token received. You can now return to ONLYOFFICE.</p>
<script>
(function () {
    var hash = new URLSearchParams(window.location.hash.slice(1));
    var search = new URLSearchParams(window.location.search);
    var token = hash.get("access_token") || search.get("code");
    if (token) {
        fetch("/token", { method: "POST", headers: {"Content-Type": "application/json"},
            body: JSON.stringify({token: token}) });
    }
})();
</script></body></html>"#;

fn header<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request.headers().iter().find_map(|header| {
        let field: &str = header.field.as_str().as_ref();
        if field.eq_ignore_ascii_case(name) {
            Some(header.value.as_str().as_ref())
        } else {
            None
        }
    })
}

pub(crate) fn approved_origin(origin: Option<&str>) -> bool {
    let Some(origin) = origin else { return true };
    if origin == "null"
        || origin == "file://"
        || origin.starts_with("file:")
        || origin.starts_with("app:")
    {
        return true;
    }
    let Some((scheme, remainder)) = origin.split_once("://") else {
        return false;
    };
    if (!scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https"))
        || remainder.is_empty()
        || remainder.contains(['/', '?', '#', '@'])
    {
        return false;
    }
    let hostname = if let Some(address) = remainder.strip_prefix('[') {
        let Some((host, suffix)) = address.split_once(']') else {
            return false;
        };
        if !suffix.is_empty() && (!suffix.starts_with(':') || suffix[1..].parse::<u16>().is_err()) {
            return false;
        }
        host
    } else {
        let mut parts = remainder.split(':');
        let host = parts.next().unwrap_or_default();
        let port = parts.next();
        if parts.next().is_some() || port.is_some_and(|value| value.parse::<u16>().is_err()) {
            return false;
        }
        host
    };
    ["localhost", "127.0.0.1", "::1"]
        .iter()
        .any(|allowed| hostname.eq_ignore_ascii_case(allowed))
}

fn approved_host(host: Option<&str>) -> bool {
    host.unwrap_or_default()
        .split(':')
        .next()
        .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1" | ""))
}

fn send_json(request: Request, status: u16, value: Value) {
    let body = value.to_string();
    let mut response = Response::from_string(body)
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes("Content-Type", "application/json; charset=utf-8").unwrap())
        .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap())
        .with_header(Header::from_bytes("Access-Control-Allow-Private-Network", "true").unwrap());
    if let Some(origin) = header(&request, "Origin") {
        if approved_origin(Some(origin)) {
            response.add_header(Header::from_bytes("Access-Control-Allow-Origin", origin).unwrap());
            response.add_header(Header::from_bytes("Vary", "Origin").unwrap());
        }
    } else if approved_origin(None) {
        response.add_header(Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
    }
    let _ = request.respond(response);
}

fn reject_request(request: Request, logger: &Logger) {
    logger.event(
        "warn",
        "http.request_rejected",
        json!({
            "method": request.method().as_str(),
            "path": request.url().split(['?', '#']).next().unwrap_or(""),
            "origin": header(&request, "Origin"),
            "host": header(&request, "Host"),
        }),
    );
    send_json(request, 403, json!({"error": "origin_not_allowed"}));
}

fn send_callback(request: Request) {
    let response = Response::from_string(HTML_CALLBACK)
        .with_status_code(200)
        .with_header(Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap())
        .with_header(Header::from_bytes("Cache-Control", "no-store").unwrap());
    let _ = request.respond(response);
}

fn send_preflight(request: Request) {
    let origin = header(&request, "Origin").unwrap_or("*").to_owned();
    let mut response = Response::empty(204)
        .with_header(Header::from_bytes("Access-Control-Allow-Origin", origin.as_str()).unwrap())
        .with_header(
            Header::from_bytes("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
                .unwrap(),
        )
        .with_header(
            Header::from_bytes(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, *",
            )
            .unwrap(),
        )
        .with_header(Header::from_bytes("Access-Control-Allow-Private-Network", "true").unwrap());
    if header(&request, "Origin").is_some() {
        response.add_header(Header::from_bytes("Vary", "Origin").unwrap());
    }
    let _ = request.respond(response);
}

fn request_path(request: &Request) -> &str {
    request.url().split(['?', '#']).next().unwrap_or("")
}

fn handle_request(mut request: Request, token_path: &Path, logger: &Logger, stopping: &mut bool) {
    if request.method() == &Method::Options {
        if approved_origin(header(&request, "Origin")) {
            send_preflight(request);
        } else {
            reject_request(request, logger);
        }
        return;
    }
    if !approved_host(header(&request, "Host")) || !approved_origin(header(&request, "Origin")) {
        reject_request(request, logger);
        return;
    }
    let path = request_path(&request).to_owned();
    match (request.method().as_str(), path.as_str()) {
        ("GET", "/health") => send_json(
            request,
            200,
            json!({"service": SERVICE_NAME, "status": "ok"}),
        ),
        ("GET", "/token") => {
            let token = if token_path == crate::token::token_file_path() {
                crate::token::read_token_or_recover(
                    token_path,
                    || crate::mendeley::get_token(logger),
                    logger,
                )
            } else {
                read_token(token_path)
            };
            send_json(request, 200, json!({"token": token}));
        }
        ("GET", "/" | "/callback") => send_callback(request),
        ("POST", "/shutdown") => {
            send_json(request, 200, json!({"status": "stopping"}));
            *stopping = true;
            logger.event("info", "server.shutdown_requested", json!({"port": PORT}));
        }
        ("POST", "/token") => {
            let size = header(&request, "Content-Length")
                .and_then(|size| size.parse::<usize>().ok())
                .unwrap_or(0);
            if size == 0 || size > MAX_BODY_SIZE {
                logger.event(
                    "error",
                    "oauth.token_rejected",
                    json!({"error": "invalid body size"}),
                );
                send_json(request, 400, json!({"error": "invalid_token_payload"}));
                return;
            }
            let mut body = Vec::with_capacity(size);
            if request
                .as_reader()
                .take((MAX_BODY_SIZE + 1) as u64)
                .read_to_end(&mut body)
                .is_err()
                || body.len() != size
                || body.len() > MAX_BODY_SIZE
            {
                logger.event(
                    "error",
                    "oauth.token_rejected",
                    json!({"error": "invalid body size"}),
                );
                send_json(request, 400, json!({"error": "invalid_token_payload"}));
                return;
            }
            let payload = serde_json::from_slice::<Value>(&body);
            let token = payload.ok().and_then(|value| {
                value
                    .get("token")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });
            let Some(token) =
                token.filter(|token| !token.is_empty() && token.chars().count() <= MAX_TOKEN_SIZE)
            else {
                logger.event(
                    "error",
                    "oauth.token_rejected",
                    json!({"error": "invalid token"}),
                );
                send_json(request, 400, json!({"error": "invalid_token_payload"}));
                return;
            };
            match write_token(token_path, &token) {
                Ok(()) => {
                    logger.event("success", "oauth.token_captured", json!({}));
                    send_json(request, 200, json!({"status": "ok"}));
                }
                Err(error) => {
                    logger.event(
                        "error",
                        "oauth.token_write_failed",
                        json!({"error": error.to_string()}),
                    );
                    send_json(request, 500, json!({"error": "token_write_failed"}));
                }
            }
        }
        ("DELETE", "/token") => match clear_token(token_path) {
            Ok(()) => send_json(request, 200, json!({"status": "cleared"})),
            Err(error) => {
                logger.event(
                    "error",
                    "oauth.token_clear_failed",
                    json!({"error": error.to_string()}),
                );
                send_json(request, 500, json!({"error": "token_clear_failed"}));
            }
        },
        _ => send_json(request, 404, json!({"error": "not_found"})),
    }
}

pub(crate) fn serve(
    server: Server,
    token_path: &Path,
    logger: &Logger,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stopping = false;
    while !stopping {
        if let Some(request) = server.recv_timeout(Duration::from_millis(100))? {
            logger.event(
                "info",
                "http.request",
                json!({"method": request.method().as_str(), "path": request_path(&request)}),
            );
            handle_request(request, token_path, logger, &mut stopping);
        }
    }
    logger.event("info", "server.stopped", json!({"port": PORT}));
    Ok(())
}
