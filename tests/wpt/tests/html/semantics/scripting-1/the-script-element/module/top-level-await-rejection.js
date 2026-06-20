window.log.push("module-start");
await Promise.reject(new Error("tla rejection sentinel"));
window.log.push("unreachable");
