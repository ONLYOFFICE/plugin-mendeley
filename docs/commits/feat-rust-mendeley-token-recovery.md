# Rust Mendeley token recovery

- Discover Mendeley storage roots across environment overrides, native OS locations, packaged Linux/Windows locations, and Linux process arguments.
- Recover access tokens from cookies, Windows DPAPI-encrypted cookies, the Mendeley refresh-token endpoint, and Service Worker CacheStorage.
- Make `/token` prefer saved tokens, recover only for the default token store, and persist recovered tokens.
- Keep the Python helper unchanged as parity reference; add Rust recovery and token-persistence regression tests.
