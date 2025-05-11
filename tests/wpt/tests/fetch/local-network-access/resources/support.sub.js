// Maps protocol (without the trailing colon) and address space to port.
const SERVER_PORTS = {
  "http": {
    "local": {{ports[http][0]}},
    "private": {{ports[http-private][0]}},
    "public": {{ports[http-public][0]}},
  },
  "https": {
    "local": {{ports[https][0]}},
    "other-local": {{ports[https][1]}},
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
  static OTHER_HTTPS_LOCAL = Server.get("https", "other-local");
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
