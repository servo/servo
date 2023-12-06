globalThis.checkTask = "PASS";
await new Promise(r => setTimeout(r, 0));
globalThis.checkTask = "FAIL";
