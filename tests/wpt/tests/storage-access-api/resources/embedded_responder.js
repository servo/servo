"use strict";

test_driver.set_test_context(window.top);

let worker;

function waitForWorkerMessage(worker) {
  return new Promise(resolve => {
    const listener = (event) => {
      worker.removeEventListener("message", listener);
      resolve(event.data);
    };
    worker.addEventListener("message", listener);
  });
}

function connectAndGetRequestCookiesFrom(origin) {
  return new Promise((resolve, reject) => {
      const ws = new WebSocket(origin +'/echo-cookie');
      ws.onmessage = event => {
          const cookies = event.data;
          resolve(cookies);
          ws.onerror = undefined;
          ws.onclose = undefined;
      };
      ws.onerror = () => reject(new Error('Unexpected error event'));
      ws.onclose = evt => reject('Unexpected close event: ' + JSON.stringify(evt));
  });
}

window.addEventListener("message", async (event) => {
  function reply(data) {
    event.source.postMessage(
        {timestamp: event.data.timestamp, data}, event.origin);
  }

  switch (event.data["command"]) {
    case "hasStorageAccess":
      reply(await document.hasStorageAccess());
      break;
    case "requestStorageAccess": {
      const obtainedAccess = await document.requestStorageAccess()
        .then(() => true, () => false);
      reply(obtainedAccess);
    }
      break;
    case "write document.cookie":
      document.cookie = event.data.cookie;
      reply(undefined);
      break;
    case "document.cookie":
      reply(document.cookie);
      break;
    case "set_permission":
      await test_driver.set_permission(...event.data.args);
      reply(undefined);
      break;
    case "observe_permission_change":
      const status = await navigator.permissions.query({name: "storage-access"});
      status.addEventListener("change", (event) => {
        reply(event.target.state)
      }, { once: true });
      break;
    case "reload":
      window.location.reload();
      break;
    case "navigate":
      window.location.href = event.data.url;
      break;
    case "httpCookies":
      // The `httpCookies` variable is defined/set by
      // script-with-cookie-header.py.
      reply(httpCookies);
      break;
    case "cors fetch":
      reply(await fetch(event.data.url, {mode: 'cors', credentials: 'include'}).then((resp) => resp.text()));
      break;
    case "no-cors fetch":
      reply(await fetch(event.data.url, {mode: 'no-cors', credentials: 'include'}).then((resp) => resp.text()));
      break;
    case "start_dedicated_worker":
      worker = new Worker("embedded_worker.js");
      reply(undefined);
      break;
    case "message_worker": {
      const p = waitForWorkerMessage(worker);
      worker.postMessage(event.data.message);
      reply(await p.then(resp => resp.data))
      break;
    }
    case "get_cookie_via_websocket":{
      reply(await connectAndGetRequestCookiesFrom(event.data.origin));
      break;
    }
    default:
  }
});
