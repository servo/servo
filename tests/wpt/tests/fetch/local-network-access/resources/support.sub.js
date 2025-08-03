// Maps protocol (without the trailing colon) and address space to port.
//
// TODO(crbug.com/418737577): change keys to be consistent with new address
// space names.
const SERVER_PORTS = {
  "http": {
    "loopback": {{ports[http][0]}},
    "other-loopback": {{ports[http][1]}},
    "local": {{ports[http-local][0]}},
    "public": {{ports[http-public][0]}},
  },
  "https": {
    "loopback": {{ports[https][0]}},
    "other-loopback": {{ports[https][1]}},
    "local": {{ports[https-local][0]}},
    "public": {{ports[https-public][0]}},
  },
  "ws": {
    "loopback": {{ports[ws][0]}},
  },
  "wss": {
    "loopback": {{ports[wss][0]}},
  },
};

// A `Server` is a web server accessible by tests. It has the following shape:
//
// {
//   addressSpace: the IP address space of the server ("loopback", "local" or
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

  static HTTP_LOOPBACK = Server.get("http", "loopback");
  static OTHER_HTTP_LOOPBACK = Server.get("http", "other-loopback");
  static HTTP_LOCAL = Server.get("http", "local");
  static HTTP_PUBLIC = Server.get("http", "public");
  static HTTPS_LOOPBACK = Server.get("https", "loopback");
  static OTHER_HTTPS_LOOPBACK = Server.get("https", "other-loopback");
  static HTTPS_LOCAL = Server.get("https", "local");
  static HTTPS_PUBLIC = Server.get("https", "public");
  static WS_LOOPBACK = Server.get("ws", "loopback");
  static WSS_LOOPBACK = Server.get("wss", "loopback");
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

// Computes the URL of a target handler configured with the given options.
//
// `server` identifies the server from which to load the resource.
// `behavior` specifies the behavior of the target server. It may contain:
//   - `response`: The result of calling one of `ResponseBehavior`'s methods.
//   - `redirect`: A URL to which the target should redirect GET requests.
function resolveTargetUrl({ server, behavior }) {
  if (server === undefined) {
    throw new Error("no server specified.");
  }
  const options = {...server};
  if (behavior) {
    const { response, redirect } = behavior;
    options.searchParams = {
      ...response,
    };
    if (redirect !== undefined) {
      options.searchParams.redirect = redirect;
    }
  }

  return resolveUrl("target.py", options);
}

// Methods generate behavior specifications for how `resources/target.py`
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

// Helper function for checking results from fetch tests.
function checkTestResult(actual, expected) {
  assert_equals(actual.error, expected.error, "error mismatch");
  assert_equals(actual.ok, expected.ok, "response ok mismatch");
  assert_equals(actual.body, expected.body, "response body mismatch");

  if (expected.type !== undefined) {
    assert_equals(type, expected.type, "response type mismatch");
  }
}

// Registers an event listener that will resolve this promise when this
// window receives a message posted to it.
function futureMessage(options) {
  return new Promise(resolve => {
    window.addEventListener('message', (e) => {
      resolve(e.data);
    });
  });
};

const NavigationTestResult = {
  SUCCESS: 'loaded',
  FAILURE: 'timeout',
};

async function iframeTest(
    t, {source, target, expected, permission = 'denied'}) {
  const targetUrl =
      resolveUrl('resources/openee.html', sourceResolveOptions(target));

  const sourceUrl =
      resolveUrl('resources/iframer.html', sourceResolveOptions(source));
  sourceUrl.searchParams.set('permission', permission);
  sourceUrl.searchParams.set('url', targetUrl);

  const popup = window.open(sourceUrl);
  t.add_cleanup(() => popup.close());

  // The child frame posts a message iff it loads successfully.
  // There exists no interoperable way to check whether an iframe failed to
  // load, so we use a timeout.
  // See: https://github.com/whatwg/html/issues/125
  const result = await Promise.race([
    futureMessage().then((data) => data.message),
    new Promise((resolve) => {
      t.step_timeout(() => resolve('timeout'), 2000 /* ms */);
    }),
  ]);

  assert_equals(result, expected);
}

async function navigateTest(t, {source, target, expected}) {
  const targetUrl =
      resolveUrl('resources/openee.html', sourceResolveOptions(target));

  const sourceUrl =
      resolveUrl('resources/navigate.html', sourceResolveOptions(source));
  sourceUrl.searchParams.set('url', targetUrl);

  const popup = window.open(sourceUrl);
  t.add_cleanup(() => popup.close());

  const result = await Promise.race([
    futureMessage().then((data) => data.message),
    new Promise((resolve) => {
      t.step_timeout(() => resolve('timeout'), 2000 /* ms */);
    }),
  ]);

  assert_equals(result, expected);
}
