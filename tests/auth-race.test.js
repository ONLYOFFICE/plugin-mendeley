const assert = require("assert");
const Auth = require("../scripts/auth");
const Events = require("../scripts/events");

async function run() {
    const desktopRequests = [];
    const intervals = [];
    const storage = new Map();
    const loads = [];
    let opens = 0;
    const original = {
        window: global.window, localStorage: global.localStorage, fetch: global.fetch,
        setInterval: global.setInterval, clearInterval: global.clearInterval,
        DocumentModule: global.DocumentModule, document: global.document,
        MendeleySDK: global.MendeleySDK, XMLHttpRequest: global.XMLHttpRequest
    };
    global.localStorage = {
        getItem: key => storage.get(key) || null,
        setItem: (key, value) => storage.set(key, value),
        removeItem: key => storage.delete(key)
    };
    global.window = {
        Asc: { plugin: {} },
        MendeleyApp: {
            Auth, Events,
            Helpers: { getLastUsedStyle: () => "apa", getSettings: () => "26014", showError() {}, showLoader() {} },
            LibraryView: { loadFilteredLibrary: () => loads.push(Auth.authFlow.getToken()) },
            UiControls: { initSelectBoxes() {}, initScrollBox: () => ({}) }
        },
        open: () => { opens++; return { closed: false, close() { this.closed = true; } }; }
    };
    global.document = { getElementById: () => null, location: { protocol: "file:" } };
    global.DocumentModule = function () {};
    global.MendeleySDK = () => ({});
    global.XMLHttpRequest = function () {
        this.open = () => {};
        this.send = () => this.onerror();
    };
    global.setInterval = callback => { intervals.push(callback); return intervals.length; };
    global.clearInterval = () => {};
    global.fetch = (url, options) => {
        if (url === "./scripts/styles_inventory.json") return Promise.resolve({ json: () => Promise.resolve([]) });
        if (url.endsWith("/health")) return Promise.resolve({ ok: true, json: () => Promise.resolve({ service: "mendeley-loopback", status: "ok" }) });
        if (options && options.method === "DELETE") return Promise.resolve({ ok: true });
        if (url.endsWith("/token")) return new Promise(resolve => desktopRequests.push(resolve));
        throw new Error("Unexpected request: " + url);
    };

    try {
        require("../scripts/code");
        const app = global.window.MendeleyApp;
        const loginBtn = { classList: { contains: () => false } };
        global.document.getElementById = id => id === "loginBtn" ? loginBtn : null;
        app.documentModule.unlockManagedContentControls = () => Promise.resolve();
        global.window.Asc.plugin.init();
        assert.strictEqual(desktopRequests.length, 1);
        loginBtn.onclick({ target: loginBtn });
        await new Promise(resolve => setImmediate(resolve));
        assert.strictEqual(intervals.length, 1, "login web harus mulai polling token");
        desktopRequests[0]({ ok: true, json: () => Promise.resolve({ token: "desktop-token" }) });
        await new Promise(resolve => setImmediate(resolve));
        assert.strictEqual(Auth.getCurrentAuthState(), "login", "cek desktop tertunda tidak boleh mengambil alih login web");
        assert.strictEqual(Auth.authFlow.getToken(), null);
        assert.deepStrictEqual(loads, []);
        Auth.OAuthCallback("web-token");
        assert.strictEqual(Auth.authFlow.getToken(), "web-token");
        assert.deepStrictEqual(loads, ["web-token"]);
        storage.clear();
        global.window._activeMendToken = null;
        Auth.switchAuthState("login");
        global.window.Asc.plugin.init();
        assert.strictEqual(desktopRequests.length, 2);
        desktopRequests[1]({ ok: true, json: () => Promise.resolve({ token: "active-desktop-token" }) });
        await new Promise(resolve => setImmediate(resolve));
        assert.strictEqual(Auth.authFlow.getToken(), "active-desktop-token");
        assert.strictEqual(opens, 1, "sesi desktop aktif tidak boleh membuka web");
        storage.clear();
        global.window._activeMendToken = null;
        Auth.switchAuthState("login");
        global.window.Asc.plugin.init();
        assert.strictEqual(desktopRequests.length, 3);
        desktopRequests[2]({ ok: true, json: () => Promise.resolve({ token: null }) });
        await new Promise(resolve => setImmediate(resolve));
        assert.strictEqual(desktopRequests.length, 4);
        desktopRequests[3]({ ok: true, json: () => Promise.resolve({ token: null }) });
        await new Promise(resolve => setImmediate(resolve));
        assert.strictEqual(opens, 2, "tanpa sesi desktop, login web harus terbuka otomatis");
        assert.strictEqual(Auth.getCurrentAuthState(), "login");
        console.log("auth race: web login tetap menguasai sesi");
    } finally {
        Object.keys(original).forEach(key => { global[key] = original[key]; });
    }
}

run().catch(error => { console.error(error); process.exitCode = 1; });
