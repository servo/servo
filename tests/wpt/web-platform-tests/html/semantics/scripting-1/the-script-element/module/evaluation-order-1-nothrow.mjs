globalThis.log.push("step-1-1");
queueMicrotask(() => globalThis.log.push("microtask"));
globalThis.log.push("step-1-2");
