// Creates a new iframe in |doc|, calls |func| on it and appends it as a child
// of |doc|.
// Returns a promise that resolves to the iframe once loaded (successfully or
// not).
// The iframe is removed from |doc| once test |t| is done running.
//
// NOTE: Because iframe elements always invoke the onload event handler, even
// in case of error, we cannot wire onerror to a promise rejection. The Promise
// constructor requires users to resolve XOR reject the promise.
function appendIframeWith(t, doc, func) {
  return new Promise(resolve => {
      const child = doc.createElement("iframe");
      func(child);
      child.onload = () => { resolve(child); };
      doc.body.appendChild(child);
      t.add_cleanup(() => { doc.body.removeChild(child); });
    });
}

// Appends a child iframe to |doc| sourced from |src|.
//
// See append_child_frame_with() for more details.
function appendIframe(t, doc, src) {
  return appendIframeWith(t, doc, child => { child.src = src; });
}

// Register an event listener that will resolve this promise when this
// window receives a message posted to it.
function futureMessage() {
  return new Promise(resolve => {
      window.addEventListener("message", e => resolve(e.data));
  });
};

const Server = {
  HTTP_LOCAL: {
    port: {{ports[http][0]}},
    protocol: "http:",
  },
  HTTP_PRIVATE: {
    port: {{ports[http-private][0]}},
    protocol: "http:",
  },
  HTTP_PUBLIC: {
    port: {{ports[http-public][0]}},
    protocol: "http:",
  },
  HTTPS_LOCAL: {
    port: {{ports[https][0]}},
    protocol: "https:",
  },
  HTTPS_PRIVATE: {
    port: {{ports[https-private][0]}},
    protocol: "https:",
  },
  HTTPS_PUBLIC: {
    port: {{ports[https-public][0]}},
    protocol: "https:",
  },
  WS_LOCAL: {
    port: {{ports[ws][0]}},
    protocol: "ws:",
  },
  WSS_LOCAL: {
    port: {{ports[wss][0]}},
    protocol: "wss:",
  },
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

// Computes options to pass to `resolveUrl()` for `resources/preflight.py`.
//
// `server` identifies the server from which to load the resource.
// `behavior` specifies the behavior of the target server. It may contain:
//   - `preflight`: The result of calling one of `PreflightBehavior`'s methods.
//   - `response`: The result of calling one of `ResponseBehavior`'s methods.
function targetResolveOptions({ server, behavior }) {
  const options = {...server};
  if (behavior) {
    const { preflight, response } = behavior;
    options.searchParams = {
      ...preflight,
      ...response,
    };
  }
  return options;
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
//     // Optional. Passed to `targetResolveOptions()`.
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

  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage();

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
//     // Optional. Passed to `targetResolveOptions()`.
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

  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));

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

async function workerScriptTest(t, { source, target, expected }) {
  const sourceUrl =
      resolveUrl("resources/worker-fetcher.html", sourceResolveOptions(source));

  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));
  targetUrl.searchParams.append("body", "postMessage({ loaded: true })")
  targetUrl.searchParams.append("mime-type", "application/javascript")

  const iframe = await appendIframe(t, document, sourceUrl);
  const reply = futureMessage();

  iframe.contentWindow.postMessage({ url: targetUrl.href }, "*");

  const { error, loaded } = await reply;

  assert_equals(error, expected.error, "worker error");
  assert_equals(loaded, expected.loaded, "response loaded");
}

async function sharedWorkerScriptTest(t, { source, target, expected }) {
  const sourceUrl = resolveUrl("resources/shared-worker-fetcher.html",
                               sourceResolveOptions(source));

  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));
  targetUrl.searchParams.append(
      "body", "onconnect = (e) => e.ports[0].postMessage({ loaded: true })")

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
  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));

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
  const targetUrl =
      resolveUrl("resources/preflight.py", targetResolveOptions(target));

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
