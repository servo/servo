globalThis.checkMicrotask = "PASS";
await 0;
globalThis.checkMicrotask = "FAIL";
