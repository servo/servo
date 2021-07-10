globalThis.log.push("step-1-1");
queueMicrotask(() => log.push("microtask"));
globalThis.log.push("step-1-2");

throw new Error("error");
