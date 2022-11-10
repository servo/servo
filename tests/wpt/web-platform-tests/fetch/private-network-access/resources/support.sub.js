// Creates a new iframe in `doc`, calls `func` on it and appends it as a child
// of `doc`.
// Returns a promise that resolves to the iframe once loaded (successfully or
// not).
// The iframe is removed from `doc` once test `t` is done running.
//
// NOTE: There exists no interoperable way to check whether an iframe failed to
// load, so this should only be used when the iframe is expected to load. It
// also means we cannot wire the iframe's `error` event to a promise
// rejection. See: https://github.com/whatwg/html/issues/125
function appendIframeWith(t, doc, func) {
  return new Promise(resolve => {
      const child = doc.createElement("iframe");
      t.add_cleanup(() => child.remove());

      child.addEventListener("load", () => resolve(child), { once: true });
      func(child);
      doc.body.appendChild(child);
    });
}

// Appends a child iframe to `doc` sourced from `src`.
//
// See `appendIframeWith()` for more details.
function appendIframe(t, doc, src) {
  return appendIframeWith(t, doc, child => { child.src = src; });
}

// Registers an event listener that will resolve this promise when this
// window receives a message posted to it.
//
// `options` has the following shape:
//
//  {
//    source: If specified, this function waits for the first message from the
//      given source only, ignoring other messages.
//
//    filter: If specified, this function calls `filter` on each incoming
//      message, and resolves iff it returns true.
//  }
//
function futureMessage(options) {
  return new Promise(resolve => {
    window.addEventListener("message", (e) => {
      if (options?.source && options.source !== e.source) {
        return;
      }

      if (options?.filter && !options.filter(e.data)) {
        return;
      }

      resolve(e.data);
    });
  });
};

// Like `promise_test()`, but executes tests in parallel like `async_test()`.
//
// Cribbed from COEP tests.
function promise_test_parallel(promise, description) {
  async_test(test => {
    promise(test)
        .then(() => test.done())
        .catch(test.step_func(error => { throw error; }));
  }, description);
};

async function postMessageAndAwaitReply(target, message) {
  const reply = futureMessage({ source: target });
  target.postMessage(message, "*");
  return await reply;
}

// Maps protocol (without the trailing colon) and address space to port.
const SERVER_PORTS = {
  "http": {
    "local": {{ports[http][0]}},
    "private": {{ports[http-private][0]}},
    "public": {{ports[http-public][0]}},
  },
  "https": {
    "local": {{ports[https][0]}},
    "private": {{ports[https-private][0]}},
    "public": {{ports[https-public][0]}},
  },
  "ws": {
    "local": {{ports[ws][0]}},
  },
  "wss": {
    "local": {{ports[wss][0]}},
  },
};

// A `Server` is a web server accessible by tests. It has the following shape:
//
// {
//   addressSpace: the IP address space of the server ("local", "private" or
//     "public"),
//   name: a human-readable name for the server,
//   port: the port on which the server listens for connections,
//   protocol: the protocol (including trailing colon) spoken by the server,
// }
//
// Constants below define the available servers, which can also be accessed
// programmatically with `get()`.
class Server {
  // Maps the given `protocol` (without a trailing colon) and `addressSpace` to
  // a server. Returns null if no such server exists.
  static get(protocol, addressSpace) {
    const ports = SERVER_PORTS[protocol];
    if (ports === undefined) {
      return null;
    }

    const port = ports[addressSpace];
    if (port === undefined) {
      return null;
    }

    return {
      addressSpace,
      name: `${protocol}-${addressSpace}`,
      port,
      protocol: protocol + ':',
    };
  }

  static HTTP_LOCAL = Server.get("http", "local");
  static HTTP_PRIVATE = Server.get("http", "private");
  static HTTP_PUBLIC = Server.get("http", "public");
  static HTTPS_LOCAL = Server.get("https", "local");
  static HTTPS_PRIVATE = Server.get("https", "private");
  static HTTPS_PUBLIC = Server.get("https", "public");
  static WS_LOCAL = Server.get("ws", "local");
  static WSS_LOCAL = Server.get("wss", "local");
};

// Resolves a URL relative to the current location, returning an absolute URL.
//
// `url` specifies the relative URL, e.g. "foo.html" or "http://foo.example".
// `options`, if defined, should have the following shape:
//
//   {
//     // Optional. Overrides the protocol of the returned URL.
//     protocol,
//
//     // Optional. Overrides the port of the returned URL.
//     port,
//
//     // Extra headers.
//     headers,
//
//     // Extra search params.
//     searchParams,
//   }
//
function resolveUrl(url, options) {
  const result = new URL(url, window.location);
  if (options === undefined) {
    return result;
  }

  const { port, protocol, headers, searchParams } = options;
  if (port !== undefined) {
    result.port = port;
  }
  if (protocol !== undefined) {
    result.protocol = protocol;
  }
  if (headers !== undefined) {
    const pipes = [];
    for (key in headers) {
      pipes.push(`header(${key},${headers[key]})`);
    }
    result.searchParams.append("pipe", pipes.join("|"));
  }
  if (searchParams !== undefined) {
    for (key in searchParams) {
      result.searchParams.append(key, searchParams[key]);
    }
  }

  return result;
}

// Computes options to pass to `resolveUrl()` for a source document's URL.
//
// `server` identifies the server from which to load the document.
// `treatAsPublic`, if set to true, specifies that the source document should
// be artificially placed in the `public` address space using CSP.
function sourceResolveOptions({ server, treatAsPublic }) {
  const options = {...server};
  if (treatAsPublic) {
    options.headers = { "Content-Security-Policy": "treat-as-public-address" };
  }
  return options;
}

// Computes the URL of a preflight handler configured with the given options.
//
// `server` identifies the server from which to load the resource.
// `behavior` specifies the behavior of the target server. It may contain:
//   - `preflight`: The result of calling one of `PreflightBehavior`'s methods.
//   - `response`: The result of calling one of `ResponseBehavior`'s methods.
//   - `redirect`: A URL to which the target should redirect GET requests.
function preflightUrl({ server, behavior }) {
  const options = {...server};
  if (behavior) {
    const { preflight, response, redirect } = behavior;
    options.searchParams = {
      ...preflight,
      ...response,
    };
    if (redirect !== undefined) {
      options.searchParams.redirect = redirect;
    }
  }

  return resolveUrl("resources/preflight.py", options);
}

// Methods generate behavior specifications for how `resources/preflight.py`
// should behave upon receiving a preflight request.
const PreflightBehavior = {
  // The preflight response should fail with a non-2xx code.
  failure: () => ({}),

  // The preflight response should be missing CORS headers.
  // `uuid` should be a UUID that uniquely identifies the preflight request.
  noCorsHeader: (uuid) => ({
    "preflight-uuid": uuid,
  }),

  // The preflight response should be missing PNA headers.
  // `uuid` should be a UUID that uniquely identifies the preflight request.
  noPnaHeader: (uuid) => ({
    "preflight-uuid": uuid,
    "preflight-headers": "cors",
  }),

  // The preflight response should succeed.
  // `uuid` should be a UUID that uniquely identifies the preflight request.
  success: (uuid) => ({
    "preflight-uuid": uuid,
    "preflight-headers": "cors+pna",
  }),

  // The preflight response should succeed and allow service-worker header.
  // `uuid` should be a UUID that uniquely identifies the preflight request.
  serviceWorkerSuccess: (uuid) => ({
    "preflight-uuid": uuid,
    "preflight-headers": "cors+pna+sw",
  }),

  // The preflight response should succeed only if it is the first preflight.
  // `uuid` should be a UUID that uniquely identifies the preflight request.
  singlePreflight: (uuid) => ({
    "preflight-uuid": uuid,
    "preflight-headers": "cors+pna",
    "expect-single-preflight": true,
  }),
};

// Methods generate behavior specifications for how `resources/preflight.py`
// should behave upon receiving a regular (non-preflight) request.
const ResponseBehavior = {
  // The response should succeed without CORS headers.
  default: () => ({}),

  // The response should succeed with CORS headers.
  allowCrossOrigin: () => ({ "final-headers": "cors" }),
};

const FetchTestResult = {
  SUCCESS: {
    ok: true,
    body: "success",
  },
  OPAQUE: {
    ok: false,
    type: "opaque",
    body: "",
  },
  FAILURE: {
    error: "TypeError: Failed to fetch",
  },
};

// Runs a fetch test. Tries to fetch a given subresource from a given document.
//
// Main argument shape:
//
//   {
//     // Optional. Passed to `sourceResolveOptions()`.
//     source,
//
//     // Optional. Passed to `preflightUrl()`.
//     target,
//
//     // Optional. Passed to `fetch()`.
//     fetchOptions,
//
//     // Required. One of the values in `FetchTestResult`.
//     expected,
//   }
//
async function fetchTest(t, { source, target, fetchOptions, expected }) {
  const sourceUrl =
      resolveUrl("resources/fetcher.html", sourceResolveOptions(source));

  const targetUrl = preflightUrl(target);

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage({ source: iframe.contentWindow });

  const message = {
    url: targetUrl.href,
    options: fetchOptions,
  };
  iframe.contentWindow.postMessage(message, "*");

  const { error, ok, type, body } = await reply;

  assert_equals(error, expected.error, "error");

  assert_equals(ok, expected.ok, "response ok");
  assert_equals(body, expected.body, "response body");

  if (expected.type !== undefined) {
    assert_equals(type, expected.type, "response type");
  }
}

const XhrTestResult = {
  SUCCESS: {
    loaded: true,
    status: 200,
    body: "success",
  },
  FAILURE: {
    loaded: false,
    status: 0,
  },
};

// Runs an XHR test. Tries to fetch a given subresource from a given document.
//
// Main argument shape:
//
//   {
//     // Optional. Passed to `sourceResolveOptions()`.
//     source,
//
//     // Optional. Passed to `preflightUrl()`.
//     target,
//
//     // Optional. Method to use when sending the request. Defaults to "GET".
//     method,
//
//     // Required. One of the values in `XhrTestResult`.
//     expected,
//   }
//
async function xhrTest(t, { source, target, method, expected }) {
  const sourceUrl =
      resolveUrl("resources/xhr-sender.html", sourceResolveOptions(source));

  const targetUrl = preflightUrl(target);

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage();

  const message = {
    url: targetUrl.href,
    method: method,
  };
  iframe.contentWindow.postMessage(message, "*");

  const { loaded, status, body } = await reply;

  assert_equals(loaded, expected.loaded, "response loaded");
  assert_equals(status, expected.status, "response status");
  assert_equals(body, expected.body, "response body");
}

const IframeTestResult = {
  SUCCESS: "loaded",
  FAILURE: "timeout",
};

async function iframeTest(t, { source, target, expected }) {
  // Allows running tests in parallel.
  const uuid = token();

  const targetUrl = preflightUrl(target);
  targetUrl.searchParams.set("file", "iframed.html");
  targetUrl.searchParams.set("iframe-uuid", uuid);

  const sourceUrl =
      resolveUrl("resources/iframer.html", sourceResolveOptions(source));
  sourceUrl.searchParams.set("url", targetUrl);

  const messagePromise = futureMessage({
    filter: (data) => data.uuid === uuid,
  });
  const iframe = await appendIframe(t, document, sourceUrl);

  // The grandchild frame posts a message iff it loads successfully.
  // There exists no interoperable way to check whether an iframe failed to
  // load, so we use a timeout.
  // See: https://github.com/whatwg/html/issues/125
  const result = await Promise.race([
      messagePromise.then((data) => data.message),
      new Promise((resolve) => {
        t.step_timeout(() => resolve("timeout"), 500 /* ms */);
      }),
  ]);

  assert_equals(result, expected);
}

const iframeGrandparentTest = ({
  name,
  grandparentServer,
  child,
  grandchild,
  expected,
}) => promise_test_parallel(async (t) => {
  // Allows running tests in parallel.
  const grandparentUuid = token();
  const childUuid = token();
  const grandchildUuid = token();

  const grandparentUrl =
      resolveUrl("resources/executor.html", grandparentServer);
  grandparentUrl.searchParams.set("executor-uuid", grandparentUuid);

  const childUrl = preflightUrl(child);
  childUrl.searchParams.set("file", "executor.html");
  childUrl.searchParams.set("executor-uuid", childUuid);

  const grandchildUrl = preflightUrl(grandchild);
  grandchildUrl.searchParams.set("file", "iframed.html");
  grandchildUrl.searchParams.set("iframe-uuid", grandchildUuid);

  const iframe = await appendIframe(t, document, grandparentUrl);

  const addChild = (url) => new Promise((resolve) => {
    const child = document.createElement("iframe");
    child.src = url;
    child.addEventListener("load", () => resolve(), { once: true });
    document.body.appendChild(child);
  });

  const grandparentCtx = new RemoteContext(grandparentUuid);
  await grandparentCtx.execute_script(addChild, [childUrl]);

  // Add a blank grandchild frame inside the child.
  // Apply a timeout to this step so that failures at this step do not block the
  // execution of other tests.
  const childCtx = new RemoteContext(childUuid);
  await Promise.race([
      childCtx.execute_script(addChild, ["about:blank"]),
      new Promise((resolve, reject) => t.step_timeout(
          () => reject("timeout adding grandchild"),
          2000 /* ms */
      )),
  ]);

  const messagePromise = futureMessage({
    filter: (data) => data.uuid === grandchildUuid,
  });
  await grandparentCtx.execute_script((url) => {
    const child = window.frames[0];
    const grandchild = child.frames[0];
    grandchild.location = url;
  }, [grandchildUrl]);

  // The great-grandchild frame posts a message iff it loads successfully.
  // There exists no interoperable way to check whether an iframe failed to
  // load, so we use a timeout.
  // See: https://github.com/whatwg/html/issues/125
  const result = await Promise.race([
      messagePromise.then((data) => data.message),
      new Promise((resolve) => {
        t.step_timeout(() => resolve("timeout"), 2000 /* ms */);
      }),
  ]);

  assert_equals(result, expected);
}, name);

const WebsocketTestResult = {
  SUCCESS: "open",

  // The code is a best guess. It is not yet entirely specified, so it may need
  // to be changed in the future based on implementation experience.
  FAILURE: "close: code 1006",
};

// Runs a websocket test. Attempts to open a websocket from `source` (in an
// iframe) to `target`, then checks that the result is as `expected`.
//
// Argument shape:
//
// {
//   // Required. Passed to `sourceResolveOptions()`.
//   source,
//
//   // Required.
//   target: {
//     // Required. Target server.
//     server,
//   }
//
//   // Required. Should be one of the values in `WebsocketTestResult`.
//   expected,
// }
//
async function websocketTest(t, { source, target, expected }) {
  const sourceUrl =
      resolveUrl("resources/socket-opener.html", sourceResolveOptions(source));

  const targetUrl = resolveUrl("/echo", target.server);

  const iframe = await appendIframe(t, document, sourceUrl);

  const reply = futureMessage();
  iframe.contentWindow.postMessage(targetUrl.href, "*");

  assert_equals(await reply, expected);
}

const WorkerScriptTestResult = {
  SUCCESS: { loaded: true },
  FAILURE: { error: "unknown error" },
};

function workerScriptUrl(target) {
  const url = preflightUrl(target);

  url.searchParams.append("body", "postMessage({ loaded: true })")
  url.searchParams.append("mime-type", "application/javascript")

  return url;
}

async function workerScriptTest(t, { source, target, expected }) {
  const sourceUrl =
      resolveUrl("resources/worker-fetcher.html", sourceResolveOptions(source));

  const targetUrl = workerScriptUrl(target);

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage();

  iframe.contentWindow.postMessage({ url: targetUrl.href }, "*");

  const { error, loaded } = await reply;

  assert_equals(error, expected.error, "worker error");
  assert_equals(loaded, expected.loaded, "response loaded");
}

async function nestedWorkerScriptTest(t, { source, target, expected }) {
  const targetUrl = workerScriptUrl(target);

  const sourceUrl = resolveUrl(
      "resources/worker-fetcher.js", sourceResolveOptions(source));
  sourceUrl.searchParams.append("url", targetUrl);

  // Iframe must be same-origin with the parent worker.
  const iframeUrl = new URL("worker-fetcher.html", sourceUrl);

  const iframe = await appendIframe(t, document, iframeUrl);
  const reply = futureMessage();

  iframe.contentWindow.postMessage({ url: sourceUrl.href }, "*");

  const { error, loaded } = await reply;

  assert_equals(error, expected.error, "worker error");
  assert_equals(loaded, expected.loaded, "response loaded");
}

async function sharedWorkerScriptTest(t, { source, target, expected }) {
  const sourceUrl = resolveUrl("resources/shared-worker-fetcher.html",
                               sourceResolveOptions(source));
  const targetUrl = preflightUrl(target);
  targetUrl.searchParams.append(
      "body", "onconnect = (e) => e.ports[0].postMessage({ loaded: true })")
  targetUrl.searchParams.append("mime-type", "application/javascript")

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage();

  iframe.contentWindow.postMessage({ url: targetUrl.href }, "*");

  const { error, loaded } = await reply;

  assert_equals(error, expected.error, "worker error");
  assert_equals(loaded, expected.loaded, "response loaded");
}

// Results that may be expected in tests.
const WorkerFetchTestResult = {
  SUCCESS: { status: 200, body: "success" },
  FAILURE: { error: "TypeError" },
};

async function workerFetchTest(t, { source, target, expected }) {
  const targetUrl = preflightUrl(target);

  const sourceUrl =
      resolveUrl("resources/fetcher.js", sourceResolveOptions(source));
  sourceUrl.searchParams.append("url", targetUrl.href);

  const fetcherUrl = new URL("worker-fetcher.html", sourceUrl);

  const reply = futureMessage();
  const iframe = await appendIframe(t, document, fetcherUrl);

  iframe.contentWindow.postMessage({ url: sourceUrl.href }, "*");

  const { error, status, message } = await reply;
  assert_equals(error, expected.error, "fetch error");
  assert_equals(status, expected.status, "response status");
  assert_equals(message, expected.message, "response body");
}

async function sharedWorkerFetchTest(t, { source, target, expected }) {
  const targetUrl = preflightUrl(target);

  const sourceUrl =
      resolveUrl("resources/shared-fetcher.js", sourceResolveOptions(source));
  sourceUrl.searchParams.append("url", targetUrl.href);

  const fetcherUrl = new URL("shared-worker-fetcher.html", sourceUrl);

  const reply = futureMessage();
  const iframe = await appendIframe(t, document, fetcherUrl);

  iframe.contentWindow.postMessage({ url: sourceUrl.href }, "*");

  const { error, status, message } = await reply;
  assert_equals(error, expected.error, "fetch error");
  assert_equals(status, expected.status, "response status");
  assert_equals(message, expected.message, "response body");
}
