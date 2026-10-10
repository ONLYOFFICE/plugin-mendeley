mod http;
mod logger;
mod mendeley;
mod service;
mod token;
mod windows_cookie;

use logger::Logger;
use serde_json::json;
use std::env;

fn main() {
    let logger = match Logger::new() {
        Ok(logger) => logger,
        Err(error) => {
            eprintln!(
                "{}",
                json!({"level": "error", "event": "server.logging_failed", "data": {"error": error.to_string()}})
            );
            return;
        }
    };
    if env::args().skip(1).any(|argument| argument == "--stop") {
        service::stop(&logger);
        return;
    }
    if let Err(error) = service::run(&logger) {
        logger.event(
            "error",
            "server.failed",
            json!({"error": error.to_string()}),
        );
        std::process::exit(1);
    }
}
