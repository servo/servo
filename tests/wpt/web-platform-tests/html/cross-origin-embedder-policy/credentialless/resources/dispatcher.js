// Define an universal message passing API. It works cross-origin and across
// browsing context groups.
const dispatcher_path =
  "/html/cross-origin-embedder-policy/credentialless/resources/dispatcher.py";
const dispatcher_url = new URL(dispatcher_path, location.href).href;

// Return a promise, limiting the number of concurrent accesses to a shared
// resources to |max_concurrent_access|.
const concurrencyLimiter = (max_concurrency) => {
  let pending = 0;
  let waiting = [];
  return async (task) => {
    pending++;
    if (pending > max_concurrency)
      await new Promise(resolve => waiting.push(resolve));
    await task();
    pending--;
    waiting.shift()?.();
  };
}

// The official web-platform-test runner sometimes drop POST requests when
// too many are requested in parallel. Limiting this document to send only one
// at a time fixes the issue.
const sendLimiter = concurrencyLimiter(1);

const send = async function(uuid, message) {
  await sendLimiter(async () => {
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
