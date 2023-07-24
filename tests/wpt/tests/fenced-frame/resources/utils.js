const STORE_URL = '/fenced-frame/resources/key-value-store.py';
const BEACON_URL = '/fenced-frame/resources/automatic-beacon-store.py';
const REMOTE_EXECUTOR_URL = '/fenced-frame/resources/remote-context-executor.https.html';
const FLEDGE_BIDDING_URL = '/fenced-frame/resources/fledge-bidding-logic.js';
const FLEDGE_BIDDING_WITH_SIZE_URL = '/fenced-frame/resources/fledge-bidding-logic-with-size.js';
const FLEDGE_DECISION_URL = '/fenced-frame/resources/fledge-decision-logic.js';

// Creates a URL that includes a list of stash key UUIDs that are being used
// in the test. This allows us to generate UUIDs on the fly and let anything
// (iframes, fenced frames, pop-ups, etc...) that wouldn't have access to the
// original UUID variable know what the UUIDs are.
// @param {string} href - The base url of the page being navigated to
// @param {string list} keylist - The list of key UUIDs to be used. Note that
//                                order matters when extracting the keys
function generateURL(href, keylist) {
  const ret_url = new URL(href, location.href);
  ret_url.searchParams.append("keylist", keylist.join(','));
  return ret_url;
}

function getRemoteContextURL(origin) {
  return new URL(REMOTE_EXECUTOR_URL, origin);
}

async function runSelectRawURL(href, resolve_to_config = false) {
  try {
    await sharedStorage.worklet.addModule(
      "/shared-storage/resources/simple-module.js");
  } catch (e) {
    // Shared Storage needs to have a module added before we can operate on it.
    // It is generated on the fly with this call, and since there's no way to
    // tell through the API if a module already exists, wrap the addModule call
    // in a try/catch so that if it runs a second time in a test, it will
    // gracefully fail rather than bring the whole test down.
  }
  return await sharedStorage.selectURL(
      'test-url-selection-operation', [{url: href}], {
        data: {'mockResult': 0},
        resolveToConfig: resolve_to_config,
        keepAlive: true
      });
}

// Similar to generateURL, but creates
// 1. An urn:uuid if `resolve_to_config` is false.
// 2. A fenced frame config object if `resolve_to_config` is true.
// This relies on a mock Shared Storage auction, since it is the simplest
// WP-exposed way to turn a url into an urn:uuid or a fenced frame config.
// Note: this function, unlike generateURL, is asynchronous and needs to be
// called with an await operator.
// @param {string} href - The base url of the page being navigated to
// @param {string list} keylist - The list of key UUIDs to be used. Note that
//                                order matters when extracting the keys
// @param {boolean} [resolve_to_config = false] - Determines whether the result
//                                                of `sharedStorage.selectURL()`
//                                                is an urn:uuid or a fenced
//                                                frame config.
// Note:
// 1. There is a limit of 3 calls per origin per pageload for
// `sharedStorage.selectURL()`, so `runSelectURL()` must also respect this
// limit.
// 2. If `resolve_to_config` is true, blink feature `FencedFramesAPIChanges`
// needs to be enabled for `selectURL()` to return a fenced frame config.
// Otherwise `selectURL()` will fall back to the old behavior that returns an
// urn:uuid.
async function runSelectURL(href, keylist = [], resolve_to_config = false) {
  const full_url = generateURL(href, keylist);
  return await runSelectRawURL(full_url, resolve_to_config);
}

async function generateURNFromFledgeRawURL(href,
  nested_urls,
  resolve_to_config = false,
  ad_with_size = false,
  requested_size = null) {
  const bidding_token = token();
  const seller_token = token();

  const ad_components_list = nested_urls.map((url) => {
    return ad_with_size ?
      { renderUrl: url, sizeGroup: "group1" } :
      { renderUrl: url }
  });

  const interestGroup = ad_with_size ?
    {
      name: 'testAd1',
      owner: location.origin,
      biddingLogicUrl: new URL(FLEDGE_BIDDING_WITH_SIZE_URL, location.origin),
      ads: [{ renderUrl: href, sizeGroup: "group1", bid: 1 }],
      userBiddingSignals: { biddingToken: bidding_token },
      trustedBiddingSignalsKeys: ['key1'],
      adComponents: ad_components_list,
      adSizes: { "size1": { width: "100px", height: "50px" } },
      sizeGroups: { "group1": ["size1"] }
    } :
    {
      name: 'testAd1',
      owner: location.origin,
      biddingLogicUrl: new URL(FLEDGE_BIDDING_URL, location.origin),
      ads: [{ renderUrl: href, bid: 1 }],
      userBiddingSignals: { biddingToken: bidding_token },
      trustedBiddingSignalsKeys: ['key1'],
      adComponents: ad_components_list,
    };

  // Pick an arbitrarily high duration to guarantee that we never leave the
  // ad interest group while the test runs.
  navigator.joinAdInterestGroup(interestGroup, /*durationSeconds=*/3000000);

  let auctionConfig = {
    seller: location.origin,
    interestGroupBuyers: [location.origin],
    decisionLogicUrl: new URL(FLEDGE_DECISION_URL, location.origin),
    auctionSignals: {biddingToken: bidding_token, sellerToken: seller_token},
    resolveToConfig: resolve_to_config
  };
  if (requested_size) {
    auctionConfig['requestedSize'] = {width: requested_size[0], height: requested_size[1]};
  }
  return navigator.runAdAuction(auctionConfig);
}

// Similar to runSelectURL, but uses FLEDGE instead of Shared Storage as the
// auctioning tool.
// Note: this function, unlike generateURL, is asynchronous and needs to be
// called with an await operator. @param {string} href - The base url of the
// page being navigated to @param {string list} keylist - The list of key UUIDs
// to be used. Note that order matters when extracting the keys
// @param {string} href - The base url of the page being navigated to
// @param {string list} keylist - The list of key UUIDs to be used. Note that
//                                order matters when extracting the keys
// @param {string list} nested_urls - A list of urls that will eventually become
//                                    the nested configs/ad components
// @param {boolean} [resolve_to_config = false] - Determines whether the result
//                                                of `navigator.runAdAuction()`
//                                                is an urn:uuid or a fenced
//                                                frame config.
// @param {boolean} [ad_with_size = false] - Determines whether the auction is
//                                           run with ad sizes specified.
async function generateURNFromFledge(href, keylist, nested_urls=[], resolve_to_config = false, ad_with_size = false, requested_size = null) {
  const full_url = generateURL(href, keylist);
  return generateURNFromFledgeRawURL(full_url, nested_urls, resolve_to_config, ad_with_size, requested_size);
}

// Extracts a list of UUIDs from the from the current page's URL.
// @returns {string list} - The list of UUIDs extracted from the page. This can
//                          be read into multiple variables using the
//                          [key1, key2, etc...] = parseKeyList(); pattern.
function parseKeylist() {
  const url = new URL(location.href);
  const keylist = url.searchParams.get("keylist");
  return keylist.split(',');
}

// Converts a same-origin URL to a cross-origin URL
// @param {URL} url - The URL object whose origin is being converted
// @param {boolean} [https=true] - Whether or not to use the HTTPS origin
//
// @returns {URL} The new cross-origin URL
function getRemoteOriginURL(url, https=true) {
  const same_origin = location.origin;
  const cross_origin = https ? get_host_info().HTTPS_REMOTE_ORIGIN
      : get_host_info().HTTP_REMOTE_ORIGIN;
  return new URL(url.toString().replace(same_origin, cross_origin));
}

// Builds a URL to be used as a remote context executor.
function generateRemoteContextURL(headers, origin) {
  // Generate the unique id for the parent/child channel.
  const uuid = token();

  // Use the absolute path of the remote context executor source file, so that
  // nested contexts will work.
  const url = getRemoteContextURL(origin ? origin : location.origin);
  url.searchParams.append('uuid', uuid);

  // Add the header to allow loading in a fenced frame.
  headers.push(["Supports-Loading-Mode", "fenced-frame"]);

  // Transform the headers into the expected format.
  // https://web-platform-tests.org/writing-tests/server-pipes.html#headers
  function escape(s) {
    return s.replace('(', '\\(').replace(')', '\\)');
  }
  const formatted_headers = headers.map((header) => {
    return `header(${escape(header[0])}, ${escape(header[1])})`;
  });
  url.searchParams.append('pipe', formatted_headers.join('|'));

  return [uuid, url];
}

function buildRemoteContextForObject(object, uuid, html) {
  // https://github.com/web-platform-tests/wpt/blob/master/common/dispatcher/README.md
  const context = new RemoteContext(uuid);
  if (html) {
    context.execute_script(
      (html_source) => {
        document.body.insertAdjacentHTML('beforebegin', html_source);
      },
    [html]);
  }

  // We need a little bit of boilerplate in the handlers because Proxy doesn't
  // work so nicely with HTML elements.
  const handler = {
    get: (target, key) => {
      if (key == "execute") {
        return context.execute_script;
      }
      if (key == "element") {
        return object;
      }
      if (key in target) {
        return target[key];
      }
      return context[key];
    },
    set: (target, key, value) => {
      target[key] = value;
      return value;
    }
  };

  const proxy = new Proxy(object, handler);
  return proxy;
}

// Attaches an object that waits for scripts to execute from RemoteContext.
// (In practice, this is either a frame or a window.)
// Returns a proxy for the object that first resolves to the object itself,
// then resolves to the RemoteContext if the property isn't found.
// The proxy also has an extra attribute `execute`, which is an alias for the
// remote context's `execute_script(fn, args=[])`.
function attachContext(object_constructor, html, headers, origin) {
  const [uuid, url] = generateRemoteContextURL(headers, origin);
  const object = object_constructor(url);
  return buildRemoteContextForObject(object, uuid, html);
}

// TODO(crbug.com/1347953): Update this function to also test
// `sharedStorage.selectURL()` that returns a fenced frame config object.
// This should be done after fixing the following flaky tests that use this
// function.
// 1. crbug.com/1372536: resize-lock-input.https.html
// 2. crbug.com/1394559: unfenced-top.https.html
async function attachOpaqueContext(generator_api, resolve_to_config, ad_with_size, requested_size, object_constructor, html, headers, origin) {
  const [uuid, url] = generateRemoteContextURL(headers, origin);
  const id = await (generator_api == 'fledge' ? generateURNFromFledge(url, [], [], resolve_to_config, ad_with_size, requested_size) : runSelectURL(url, [], resolve_to_config));
  const object = object_constructor(id);
  return buildRemoteContextForObject(object, uuid, html);
}

function attachPotentiallyOpaqueContext(generator_api, resolve_to_config, ad_with_size, requested_size, frame_constructor, html, headers, origin) {
  generator_api = generator_api.toLowerCase();
  if (generator_api == 'fledge' || generator_api == 'sharedstorage') {
    return attachOpaqueContext(generator_api, resolve_to_config, ad_with_size, requested_size, frame_constructor, html, headers, origin);
  } else {
    return attachContext(frame_constructor, html, headers, origin);
  }
}

function attachFrameContext(element_name, generator_api, resolve_to_config, ad_with_size, requested_size, html, headers, attributes, origin) {
  frame_constructor = (id) => {
    frame = document.createElement(element_name);
    attributes.forEach(attribute => {
      frame.setAttribute(attribute[0], attribute[1]);
    });
    if (element_name == "iframe") {
      frame.src = id;
    } else if (id instanceof FencedFrameConfig) {
      frame.config = id;
    } else {
      const config = new FencedFrameConfig(id);
      frame.config = config;
    }
    document.body.append(frame);
    return frame;
  };
  return attachPotentiallyOpaqueContext(generator_api, resolve_to_config, ad_with_size, requested_size, frame_constructor, html, headers, origin);
}

function replaceFrameContext(frame_proxy, {generator_api="", resolve_to_config=false, ad_with_size=false, requested_size=null, html="", headers=[], origin=""}={}) {
  frame_constructor = (id) => {
    if (frame_proxy.element.nodeName == "IFRAME") {
      frame_proxy.element.src = id;
    } else if (id instanceof FencedFrameConfig) {
      frame_proxy.element.config = id;
    } else {
      const config = new FencedFrameConfig(id);
      frame_proxy.element.config = config;
    }
    return frame_proxy.element;
  };
  return attachPotentiallyOpaqueContext(generator_api, resolve_to_config, ad_with_size, requested_size, frame_constructor, html, headers, origin);
}

// Attach a fenced frame that waits for scripts to execute.
// Takes as input a(n optional) dictionary of configs:
// - generator_api: the name of the API that should generate the urn/config.
//    Supports (case-insensitive) "fledge" and "sharedstorage", or any other
//    value as a default.
//    If you generate a urn, then you need to await the result of this function.
// - resolve_to_config: whether a config should be used. (currently only works
//    for FLEDGE and sharedStorage generator_api)
// - ad_with_size: whether an ad auction is run with size specified for the ads
//    and ad components. (currently only works for FLEDGE)
// - requested_size: A 2-element list with the width and height for
//    requestedSize in the FLEDGE auction config.
// - html: extra HTML source code to inject into the loaded frame
// - headers: an array of header pairs [[key, value], ...]
// - attributes: an array of attribute pairs to set on the frame [[key, value], ...]
// - origin: origin of the url, default to location.origin if not set
// Returns a proxy that acts like the frame HTML element, but with an extra
// function `execute`. See `attachFrameContext` or the README for more details.
function attachFencedFrameContext({generator_api="", resolve_to_config=false, ad_with_size=false, requested_size=null, html = "", headers=[], attributes=[], origin=""}={}) {
  return attachFrameContext('fencedframe', generator_api, resolve_to_config, ad_with_size, requested_size, html, headers, attributes, origin);
}

// Attach an iframe that waits for scripts to execute.
// See `attachFencedFrameContext` for more details.
function attachIFrameContext({generator_api="", html="", headers=[], attributes=[], origin=""}={}) {
  return attachFrameContext('iframe', generator_api, resolve_to_config=false, ad_with_size=false, requested_size=null, html, headers, attributes, origin);
}

// Open a window that waits for scripts to execute.
// Returns a proxy that acts like the window object, but with an extra
// function `execute`. See `attachContext` for more details.
function attachWindowContext({target="_blank", html="", headers=[], origin=""}={}) {
  window_constructor = (url) => {
    return window.open(url, target);
  }

  return attachContext(window_constructor, html, headers, origin);
}

// Converts a key string into a key uuid using a cryptographic hash function.
// This function only works in secure contexts (HTTPS).
async function stringToStashKey(string) {
  // Compute a SHA-256 hash of the input string, and convert it to hex.
  const data = new TextEncoder().encode(string);
  const digest = await crypto.subtle.digest('SHA-256', data);
  const digest_array = Array.from(new Uint8Array(digest));
  const digest_as_hex = digest_array.map(b => b.toString(16).padStart(2, '0')).join('');

  // UUIDs are structured as 8X-4X-4X-4X-12X.
  // Use the first 32 hex digits and ignore the rest.
  const digest_slices = [digest_as_hex.slice(0,8),
                         digest_as_hex.slice(8,12),
                         digest_as_hex.slice(12,16),
                         digest_as_hex.slice(16,20),
                         digest_as_hex.slice(20,32)];
  return digest_slices.join('-');
}

// Create a fenced frame. Then navigate it using the given `target`, which can
// be either an urn:uuid or a fenced frame config object.
function attachFencedFrame(target) {
  assert_implements(
      window.HTMLFencedFrameElement,
      'The HTMLFencedFrameElement should be exposed on the window object');

  const fenced_frame = document.createElement('fencedframe');

  if (target instanceof FencedFrameConfig) {
    fenced_frame.config = target;
  } else {
    const config = new FencedFrameConfig(target);
    fenced_frame.config = config;
  }

  document.body.append(fenced_frame);
  return fenced_frame;
}

function attachIFrame(url) {
  const iframe = document.createElement('iframe');
  iframe.src = url;
  document.body.append(iframe);
  return iframe;
}

// Reads the value specified by `key` from the key-value store on the server.
async function readValueFromServer(key) {
  // Resolve the key if it is a Promise.
  key = await key;

  const serverUrl = `${STORE_URL}?key=${key}`;
  const response = await fetch(serverUrl);
  if (!response.ok)
    throw new Error('An error happened in the server');
  const value = await response.text();

  // The value is not stored in the server.
  if (value === "<Not set>")
    return { status: false };

  return { status: true, value: value };
}

// Convenience wrapper around the above getter that will wait until a value is
// available on the server.
async function nextValueFromServer(key) {
  // Resolve the key if it is a Promise.
  key = await key;

  while (true) {
    // Fetches the test result from the server.
    const { status, value } = await readValueFromServer(key);
    if (!status) {
      // The test result has not been stored yet. Retry after a while.
      await new Promise(resolve => setTimeout(resolve, 20));
      continue;
    }

    return value;
  }
}

// Reads the data from the latest automatic beacon sent to the server.
async function readAutomaticBeaconDataFromServer() {
  const serverUrl = `${BEACON_URL}`;
  const response = await fetch(serverUrl);
  if (!response.ok)
    throw new Error('An error happened in the server');
  const value = await response.text();

  // The value is not stored in the server.
  if (value === "<Not set>")
    return { status: false };

  return { status: true, value: value };
}

// Convenience wrapper around the above getter that will wait until a value is
// available on the server.
async function nextAutomaticBeacon() {
  while (true) {
    // Fetches the test result from the server.
    const { status, value } = await readAutomaticBeaconDataFromServer();
    if (!status) {
      // The test result has not been stored yet. Retry after a while.
      await new Promise(resolve => setTimeout(resolve, 20));
      continue;
    }

    return value;
  }
}

// Writes `value` for `key` in the key-value store on the server.
async function writeValueToServer(key, value, origin = '') {
  // Resolve the key if it is a Promise.
  key = await key;

  const serverUrl = `${origin}${STORE_URL}?key=${key}&value=${value}`;
  await fetch(serverUrl, {"mode": "no-cors"});
}

// Simulates a user gesture.
async function simulateGesture() {
  // Wait until the window size is initialized.
  while (window.innerWidth == 0) {
    await new Promise(resolve => requestAnimationFrame(resolve));
  }
  await test_driver.bless('simulate gesture');
}

// Fenced frames are always put in the public IP address space which is the
// least privileged. In case a navigation to a local data: URL or blob: URL
// resource is allowed, they would only be able to fetch things that are *also*
// in the public IP address space. So for the document described by these local
// URLs, we'll set them up to only communicate back to the outer page via
// resources obtained in the public address space.
function createLocalSource(key, url) {
  return `
    <head>
      <script src="${url}"><\/script>
    </head>
    <body>
      <script>
        writeValueToServer("${key}", "LOADED", /*origin=*/"${url.origin}");
      <\/script>
    </body>
  `;
}

function setupCSP(csp, second_csp=null) {
  let meta = document.createElement('meta');
  meta.httpEquiv = "Content-Security-Policy";
  meta.content = "fenced-frame-src " + csp;
  document.head.appendChild(meta);

  if (second_csp != null) {
    let second_meta = document.createElement('meta');
    second_meta.httpEquiv = "Content-Security-Policy";
    second_meta.content = "frame-src " + second_csp;
    document.head.appendChild(second_meta);
  }
}
