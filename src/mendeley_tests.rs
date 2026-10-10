use super::{add_dir, cache_token, extract_cookie_token};
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "mendeley-recovery-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn storage_roots_are_canonical_and_deduplicated() {
    let root = temporary_directory();
    fs::create_dir_all(&root).unwrap();
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    add_dir(&root, &mut paths, &mut seen);
    add_dir(root.join("."), &mut paths, &mut seen);
    assert_eq!(paths, vec![fs::canonicalize(&root).unwrap()]);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_access_token_cookie_is_returned_without_network_refresh() {
    let root = temporary_directory();
    fs::create_dir_all(&root).unwrap();
    let database = root.join("Cookies");
    let connection = Connection::open(&database).unwrap();
    connection
        .execute(
            "CREATE TABLE cookies (name TEXT, value TEXT, encrypted_value BLOB)",
            [],
        )
        .unwrap();
    connection.execute("INSERT INTO cookies VALUES ('accessToken', 'MS,fake-token-value-with-length-over-20', x'')", []).unwrap();
    drop(connection);
    assert_eq!(
        extract_cookie_token(&database).as_deref(),
        Some("MS,fake-token-value-with-length-over-20")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn service_worker_cache_recovers_bearer_token_and_rejects_short_values() {
    let root = temporary_directory();
    let cache = root.join("Service Worker/CacheStorage/test");
    fs::create_dir_all(&cache).unwrap();
    let file = cache.join("cache_data_0");
    fs::write(
        &file,
        b"Bearer short Bearer MSwx_fake_cached_token_1234567890_value!",
    )
    .unwrap();
    assert_eq!(
        cache_token(&root).as_deref(),
        Some("MSwx_fake_cached_token_1234567890_value")
    );
    fs::remove_dir_all(root).unwrap();
}
