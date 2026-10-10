import importlib.util
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest
from urllib.error import HTTPError
from urllib.request import Request, urlopen


MODULE_PATH = Path(__file__).parents[1] / "scripts" / "mendeley-loopback-server.py"
SPEC = importlib.util.spec_from_file_location("mendeley_loopback_server", MODULE_PATH)
SERVER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SERVER)


class LoopbackServerTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.token_path = Path(self.temp_dir.name) / "token.json"
        self.server = SERVER.create_server("127.0.0.1", 0, self.token_path)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base_url = "http://127.0.0.1:{}".format(self.server.server_port)

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=2)
        self.temp_dir.cleanup()

    def test_health_allows_only_loopback_web_origins(self):
        request = Request(
            self.base_url + "/health",
            headers={"Origin": "http://localhost:3000"},
        )
        with urlopen(request) as response:
            self.assertEqual(response.status, 200)
            self.assertEqual(response.headers["Access-Control-Allow-Origin"], "http://localhost:3000")

        request = Request(
            self.base_url + "/health",
            headers={"Origin": "https://attacker.invalid"},
        )
        with self.assertRaises(HTTPError) as error:
            urlopen(request)
        self.assertEqual(error.exception.code, 403)
        error.exception.close()
    def test_file_origin_is_allowed_for_onlyoffice_plugin(self):
        request = Request(
            self.base_url + "/health",
            headers={"Origin": "file://"},
        )
        with urlopen(request) as response:
            self.assertEqual(response.status, 200)
            self.assertEqual(response.headers["Access-Control-Allow-Origin"], "file://")
        preflight = Request(
            self.base_url + "/token",
            headers={"Origin": "file://", "Access-Control-Request-Method": "DELETE"},
            method="OPTIONS",
        )
        with urlopen(preflight) as response:
            self.assertEqual(response.status, 204)
            self.assertEqual(response.headers["Access-Control-Allow-Origin"], "file://")

        request = Request(
            self.base_url + "/token",
            headers={"Origin": "file://"},
            method="DELETE",
        )
        with urlopen(request) as response:
            self.assertEqual(json.load(response), {"status": "cleared"})

    def test_loopback_origin_can_preflight_and_clear_token(self):
        SERVER.write_token(self.token_path, "stale-token")
        origin = "http://localhost:3000"

        preflight = Request(
            self.base_url + "/token",
            headers={
                "Origin": origin,
                "Access-Control-Request-Method": "DELETE",
            },
            method="OPTIONS",
        )
        with urlopen(preflight) as response:
            self.assertEqual(response.status, 204)
            self.assertEqual(response.headers["Access-Control-Allow-Origin"], origin)
            self.assertIn("DELETE", response.headers["Access-Control-Allow-Methods"])

        request = Request(
            self.base_url + "/token",
            headers={"Origin": origin},
            method="DELETE",
        )
        with urlopen(request) as response:
            self.assertEqual(json.load(response), {"status": "cleared"})
        self.assertFalse(self.token_path.exists())


    def test_health_endpoint_identifies_helper_for_launcher_reuse(self):
        with urlopen(self.base_url + "/health") as response:
            self.assertEqual(response.status, 200)
            self.assertEqual(json.load(response), {"service": "mendeley-loopback", "status": "ok"})

    def test_token_endpoint_persists_owner_only_token(self):
        request = Request(
            self.base_url + "/token",
            data=json.dumps({"token": "secret-token"}).encode(),
            headers={"Content-Type": "application/json", "Origin": "null"},
            method="POST",
        )
        with urlopen(request) as response:
            self.assertEqual(json.load(response), {"status": "ok"})
        self.assertEqual(json.loads(self.token_path.read_text())["token"], "secret-token")
        if os.name != "nt":
            self.assertEqual(self.token_path.stat().st_mode & 0o777, 0o600)

    def test_token_endpoint_rejects_unapproved_origin(self):
        request = Request(
            self.base_url + "/token",
            data=b'{"token":"secret-token"}',
            headers={"Content-Type": "application/json", "Origin": "https://attacker.invalid"},
            method="POST",
        )
        with self.assertRaises(HTTPError) as error:
            urlopen(request)
        self.assertEqual(error.exception.code, 403)
        error.exception.close()
        self.assertFalse(self.token_path.exists())

    def test_delete_endpoint_clears_previous_login_token(self):
        SERVER.write_token(self.token_path, "stale-token")
        request = Request(
            self.base_url + "/token",
            headers={"Origin": "null"},
            method="DELETE",
        )
        with urlopen(request) as response:
            self.assertEqual(json.load(response), {"status": "cleared"})
        self.assertFalse(self.token_path.exists())
    def test_startup_reuses_running_mendeley_server(self):
        self.assertIsNone(
            SERVER.start_or_reuse_server("127.0.0.1", self.server.server_port, self.token_path)
        )

    def test_stop_command_stops_only_healthy_mendeley_helper(self):
        self.assertTrue(SERVER.stop_server(self.server.server_port))
        self.thread.join(timeout=2)
        self.assertFalse(self.thread.is_alive())

    def test_auto_sync_from_mendeley_app_cookies(self):
        import sqlite3
        fake_cookie_dir = Path(self.temp_dir.name) / "config" / "Mendeley Reference Manager"
        fake_cookie_dir.mkdir(parents=True, exist_ok=True)
        fake_db = fake_cookie_dir / "Cookies"
        con = sqlite3.connect(fake_db)
        con.execute("CREATE TABLE cookies (name TEXT, value TEXT, encrypted_value BLOB)")
        con.execute("INSERT INTO cookies VALUES ('accessToken', 'MS,fake-token-value-with-length-over-20', x'')")
        con.commit()
        con.close()

        orig_environ = os.environ.get("XDG_CONFIG_HOME")
        os.environ["XDG_CONFIG_HOME"] = str(Path(self.temp_dir.name) / "config")
        try:
            token = SERVER.get_token_from_mendeley_app()
            self.assertEqual(token, "MS,fake-token-value-with-length-over-20")
        finally:
            if orig_environ is not None:
                os.environ["XDG_CONFIG_HOME"] = orig_environ
            else:
                os.environ.pop("XDG_CONFIG_HOME", None)

    def test_refresh_cookie_extraction_imports_urllib_request(self):
        import sqlite3

        cookie_db = Path(self.temp_dir.name) / "Cookies"
        with sqlite3.connect(cookie_db) as connection:
            connection.execute("CREATE TABLE cookies (name TEXT, value TEXT, encrypted_value BLOB)")
            connection.execute("INSERT INTO cookies VALUES ('refreshToken', 'fake-refresh-cookie', x'')")

        request_module = SERVER.urllib_request
        original_urlopen = request_module.urlopen

        class Response:
            status = 200

            def __enter__(self):
                return self

            def __exit__(self, *args):
                return False

            def read(self):
                return b'{"access_token":"MS_fake_access_token_value_123456789"}'

        request_module.urlopen = lambda request, timeout: Response()
        del SERVER.urllib.request
        try:
            token = SERVER._extract_token_from_cookies(cookie_db)
        finally:
            SERVER.urllib.request = request_module
            request_module.urlopen = original_urlopen

        self.assertEqual(token, "MS_fake_access_token_value_123456789")

    def test_auto_sync_from_cache_storage(self):
        fake_cache_dir = Path(self.temp_dir.name) / "config" / "Mendeley Reference Manager" / "Service Worker" / "CacheStorage" / "dummy"
        fake_cache_dir.mkdir(parents=True, exist_ok=True)
        fake_cache_file = fake_cache_dir / "cache_data_0"
        fake_cache_file.write_bytes(b"Header data Bearer MSwx_fake_cached_token_1234567890_value Trailer data")

        orig_environ = os.environ.get("XDG_CONFIG_HOME")
        os.environ["XDG_CONFIG_HOME"] = str(Path(self.temp_dir.name) / "config")
        try:
            token = SERVER.get_token_from_mendeley_app()
            self.assertEqual(token, "MSwx_fake_cached_token_1234567890_value")
        finally:
            if orig_environ is not None:
                os.environ["XDG_CONFIG_HOME"] = orig_environ
            else:
                os.environ.pop("XDG_CONFIG_HOME", None)


if __name__ == "__main__":
    unittest.main()
