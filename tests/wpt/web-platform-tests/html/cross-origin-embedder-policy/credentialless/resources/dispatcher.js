// Define an universal message passing API. It works cross-origin and across
// browsing context groups.
const dispatcher_path =
  "/html/cross-origin-embedder-policy/credentialless/resources/dispatcher.py";
const dispatcher_url = new URL(dispatcher_path, location.href).href;

const send = async function(uuid, message) {
  // The official web-platform-test runner sometimes drop POST requests when
  // many are requested in parallel. Using a lock fixes the issue.
  await navigator.locks.request("dispatcher_send", async lock => {
    await fetch(dispatcher_url + `?uuid=${uuid}`, {
      method: 'POST',
      body: message
    });
  });
}

const receive = async function(uuid) {
  while(1) {
    let response = await fetch(dispatcher_url + `?uuid=${uuid}`);
    let data = await response.text();
    if (data != 'not ready')
      return data;
    await new Promise(r => setTimeout(r, 10 + 100*Math.random()));
  }
}

// Returns an URL. When called, the server sends toward the `uuid` queue the
// request headers. Useful for determining if something was requested with
// Cookies.
const showRequestHeaders= function(origin, uuid) {
  return origin + dispatcher_path + `?uuid=${uuid}&show-headers`;
}
