import { createVapid } from "./vapid.js";

export function permissionTest(origin, sender, registration) {
  function ping(message) {
    if (!globalThis.WorkerGlobalScope) {
      window.top.postMessage(message, origin);
    } else {
      globalThis.postMessage(message);
    }
  }

  (async () => {
    const vapid = await createVapid();
    const subscribed = await registration.pushManager.subscribe({
      applicationServerKey: vapid.publicKey
    }).then(() => true, _ => false)
    ping({ sender, subscribed });
  })();

  if (!globalThis.WorkerGlobalScope) {
    const workerUrl = new URL(`./permission-worker.js`, import.meta.url);
    workerUrl.searchParams.set("sender", `${sender}Worker`);
    const worker = new Worker(workerUrl, { type: "module" });
    worker.onmessage = ev => ping(ev.data, origin);
  }
}
