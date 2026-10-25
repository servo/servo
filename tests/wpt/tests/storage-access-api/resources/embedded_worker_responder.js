"use strict";

// Extends embedded_responder.js with commands to start a SharedWorker or a
// ServiceWorker from the frame and to talk to them.
const baseResponder = document.createElement("script");
baseResponder.src = "embedded_responder.js";
document.body.appendChild(baseResponder);

let sharedWorker;
let serviceWorkerRegistration;

window.addEventListener("message", async (event) => {
  function reply(data) {
    event.source.postMessage({timestamp: event.data.timestamp, data}, "*");
  }

  switch (event.data["command"]) {
  case "start_shared_worker":
    sharedWorker = new SharedWorker("embedded_shared_worker.js", {name: event.data.name});
    sharedWorker.port.start();
    reply(undefined);
    break;
  case "message_shared_worker": {
    const response = new Promise((resolve) => {
      sharedWorker.port.addEventListener("message", (e) => resolve(e.data), {once: true});
    });
    sharedWorker.port.postMessage(event.data.message);
    reply((await response).data);
    break;
  }
  case "start_service_worker": {
    serviceWorkerRegistration = await navigator.serviceWorker.register(
        "embedded_service_worker.js", {scope: event.data.scope});
    await new Promise((resolve) => {
      const worker = serviceWorkerRegistration.installing || serviceWorkerRegistration.waiting || serviceWorkerRegistration.active;
      if (worker.state === "activated") {
        resolve();
        return;
      }
      worker.addEventListener("statechange", () => {
        if (worker.state === "activated")
          resolve();
      });
    });
    reply(undefined);
    break;
  }
  case "message_service_worker": {
    const response = new Promise((resolve) => {
      navigator.serviceWorker.addEventListener("message", (e) => resolve(e.data), {once: true});
    });
    serviceWorkerRegistration.active.postMessage(event.data.message);
    reply((await response).data);
    break;
  }
  case "unregister_service_worker":
    if (serviceWorkerRegistration)
      await serviceWorkerRegistration.unregister();
    reply(undefined);
    break;
  default:
  }
});
