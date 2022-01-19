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
    let result = await task();
    pending--;
    waiting.shift()?.();
    return result;
  };
}

// Wait for a random amount of time in the range [10ms,100ms].
const randomDelay = () => {
  return new Promise(resolve => setTimeout(resolve, 10 + 90*Math.random()));
}

// Sending too many requests in parallel causes congestion. Limiting it improves
// throughput.
//
// Note: The following table has been determined on the test:
// ../cache-storage.tentative.https.html
// using Chrome with a 64 core CPU / 64GB ram, in release mode:
// ┌───────────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬────┐
// │concurrency│ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 10│ 15│ 20│ 30│ 50│ 100│
// ├───────────┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼───┼────┤
// │time (s)   │ 54│ 38│ 31│ 29│ 26│ 24│ 22│ 22│ 22│ 22│ 34│ 36 │
// └───────────┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴────┘
const limiter = concurrencyLimiter(6);

const send = async function(uuid, message) {
  await limiter(async () => {
    // Requests might be dropped. Retry until getting a confirmation it has been
    // processed.
    while(1) {
      try {
        let response = await fetch(dispatcher_url + `?uuid=${uuid}`, {
          method: 'POST',
          body: message
        })
        if (await response.text() == "done")
          return;
      } catch (fetch_error) {}
      await randomDelay();
    };
  });
}

const receive = async function(uuid) {
  while(1) {
    let data = "not ready";
    try {
      data = await limiter(async () => {
        let response = await fetch(dispatcher_url + `?uuid=${uuid}`);
        return await response.text();
      });
    } catch (fetch_error) {}

    if (data == "not ready") {
      await randomDelay();
      continue;
    }

    return data;
  }
}

// Returns an URL. When called, the server sends toward the `uuid` queue the
// request headers. Useful for determining if something was requested with
// Cookies.
const showRequestHeaders= function(origin, uuid) {
  return origin + dispatcher_path + `?uuid=${uuid}&show-headers`;
}
