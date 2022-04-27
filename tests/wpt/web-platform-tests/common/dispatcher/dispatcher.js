// Define a universal message passing API. It works cross-origin and across
// browsing context groups.
const dispatcher_path = "/common/dispatcher/dispatcher.py";
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
const showRequestHeaders = function(origin, uuid) {
  return origin + dispatcher_path + `?uuid=${uuid}&show-headers`;
}

// Same as above, except for the response is cacheable.
const cacheableShowRequestHeaders = function(origin, uuid) {
  return origin + dispatcher_path + `?uuid=${uuid}&cacheable&show-headers`;
}

// This script requires
// - `/common/utils.js` for `token()`.

// Represents a remote executor. For more detailed explanation see `README.md`.
class RemoteContext {
  // `uuid` is a UUID string that identifies the remote context and should
  // match with the `uuid` parameter of the URL of the remote context.
  constructor(uuid) {
    this.context_id = uuid;
  }

  // Evaluates the script `expr` on the executor.
  // - If `expr` is evaluated to a Promise that is resolved with a value:
  //   `execute_script()` returns a Promise resolved with the value.
  // - If `expr` is evaluated to a non-Promise value:
  //   `execute_script()` returns a Promise resolved with the value.
  // - If `expr` throws an error or is evaluated to a Promise that is rejected:
  //   `execute_script()` returns a rejected Promise with the error's
  //   `message`.
  //   Note that currently the type of error (e.g. DOMException) is not
  //   preserved, except for `TypeError`.
  // The values should be able to be serialized by JSON.stringify().
  async execute_script(fn, args) {
    const receiver = token();
    await this.send({receiver: receiver, fn: fn.toString(), args: args});
    const response = JSON.parse(await receive(receiver));
    if (response.status === 'success') {
      return response.value;
    }

    // exception
    if (response.name === 'TypeError') {
      throw new TypeError(response.value);
    }
    throw new Error(response.value);
  }

  async send(msg) {
    return await send(this.context_id, JSON.stringify(msg));
  }
};

class Executor {
  constructor(uuid) {
    this.uuid = uuid;

    // If `suspend_callback` is not `null`, the executor should be suspended
    // when there are no ongoing tasks.
    this.suspend_callback = null;

    this.execute();
  }

  // Wait until there are no ongoing tasks nor fetch requests for polling
  // tasks, and then suspend the executor and call `callback()`.
  // Navigation from the executor page should be triggered inside `callback()`,
  // to avoid conflict with in-flight fetch requests.
  suspend(callback) {
    this.suspend_callback = callback;
  }

  resume() {
  }

  async execute() {
    while(true) {
      if (this.suspend_callback !== null) {
        this.suspend_callback();
        this.suspend_callback = null;
        // Wait for `resume()` to be called.
        await new Promise(resolve => this.resume = resolve);

        // Workaround for https://crbug.com/1244230.
        // Without this workaround, the executor is resumed and the fetch
        // request to poll the next task is initiated synchronously from
        // pageshow event after the page restored from BFCache, and the fetch
        // request promise is never resolved (and thus the test results in
        // timeout) due to https://crbug.com/1244230. The root cause is not yet
        // known, but setTimeout() with 0ms causes the resume triggered on
        // another task and seems to resolve the issue.
        await new Promise(resolve => setTimeout(resolve, 0));

        continue;
      }

      const task = JSON.parse(await receive(this.uuid));

      let response;
      try {
        const value = await eval(task.fn).apply(null, task.args);
        response = JSON.stringify({
          status: 'success',
          value: value
        });
      } catch(e) {
        response = JSON.stringify({
          status: 'exception',
          name: e.name,
          value: e.message
        });
      }
      await send(task.receiver, response);
    }
  }
}
