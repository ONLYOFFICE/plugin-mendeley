const assert = require("assert");
const Helpers = require("../scripts/helpers");
const Auth = require("../scripts/auth");

async function run() {
    const originalFetch = global.fetch;
    const originalWindow = global.window;
    const originalLocalStorage = global.localStorage;
    const originalShowError = Helpers.showError;
    const originalShowLoader = Helpers.showLoader;
    let displayedError = "";

    global.window = { MendeleyApp: {} };
    global.localStorage = { removeItem() {} };
    global.fetch = () => Promise.resolve({
        ok: true,
        json: () => Promise.reject(new SyntaxError("Unexpected token '<'"))
    });
    Helpers.showError = message => { displayedError = message; };
    Helpers.showLoader = () => {};

    try {
        Auth.authFlow.authenticate();
        await new Promise(resolve => setTimeout(resolve, 0));
        assert.match(displayedError, /port 8080 .*mendeley helper/i);
    } finally {
        global.fetch = originalFetch;
        global.window = originalWindow;
        global.localStorage = originalLocalStorage;
        Helpers.showError = originalShowError;
        Helpers.showLoader = originalShowLoader;
    }
}

run().catch(error => {
    console.error(error);
    process.exitCode = 1;
});
