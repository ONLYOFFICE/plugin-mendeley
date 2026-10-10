use crate::http;
use crate::logger::Logger;
use crate::token::token_file_path;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;
use tiny_http::Server;

const HOST: &str = "127.0.0.1";
pub(crate) const PORT: u16 = 8080;
pub(crate) const SERVICE_NAME: &str = "mendeley-loopback";

fn health_payload(value: &Value) -> bool {
    value == &json!({"service": SERVICE_NAME, "status": "ok"})
}

fn request(host: &str, port: u16, method: &str, path: &str) -> std::io::Result<(u16, String)> {
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(
        format!("{method} {path} HTTP/1.0\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n")
            .as_bytes(),
    )?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let (head, body) = response.split_once("\r\n\r\n").unwrap_or((&response, ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or(0);
    Ok((status, body.to_owned()))
}

fn running_service(host: &str, port: u16) -> bool {
    request(host, port, "GET", "/health")
        .ok()
        .is_some_and(|(status, body)| {
            status == 200 && serde_json::from_str(&body).is_ok_and(|value| health_payload(&value))
        })
}

pub fn stop(logger: &Logger) {
    if !running_service(HOST, PORT) {
        logger.event("info", "server.stop_not_running", json!({"port": PORT}));
        return;
    }
    match request(HOST, PORT, "POST", "/shutdown") {
        Ok((200, _)) => {
            for _ in 0..20 {
                if !running_service(HOST, PORT) {
                    logger.event("success", "server.stopped", json!({"port": PORT}));
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
            logger.event("error", "server.stop_timeout", json!({"port": PORT}));
        }
        Ok((status, _)) => logger.event(
            "error",
            "server.stop_failed",
            json!({"port": PORT, "status": status}),
        ),
        Err(error) => logger.event(
            "error",
            "server.stop_failed",
            json!({"port": PORT, "error": error.to_string()}),
        ),
    }
}

pub fn run(logger: &Logger) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = match Server::http(format!("{HOST}:{PORT}")) {
        Ok(server) => server,
        Err(error) => {
            if running_service(HOST, PORT) {
                logger.event("info", "server.reused", json!({"host": HOST, "port": PORT}));
                return Ok(());
            }
            logger.event(
                "error",
                "server.start_failed",
                json!({"host": HOST, "port": PORT, "error": error.to_string()}),
            );
            return Err(error.into());
        }
    };
    logger.event(
        "success",
        "server.started",
        json!({"host": HOST, "port": PORT}),
    );
    http::serve(server, &token_file_path(), logger)
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
