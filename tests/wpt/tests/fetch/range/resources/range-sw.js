importScripts('/resources/testharness.js');

setup({ explicit_done: true });

function assert_range_request(request, expectedRangeHeader, name) {
  assert_equals(request.headers.get('Range'), expectedRangeHeader, name);
}

async function broadcast(msg) {
  for (const client of await clients.matchAll()) {
    client.postMessage(msg);
  }
}

addEventListener('fetch', async event => {
  /** @type Request */
  const request = event.request;
  const url = new URL(request.url);
  const action = url.searchParams.get('action');

  switch (action) {
    case 'range-header-filter-test':
      rangeHeaderFilterTest(request);
      return;
    case 'range-header-passthrough-test':
      rangeHeaderPassthroughTest(event);
      return;
    case 'store-ranged-response':
      storeRangedResponse(event);
      return;
    case 'use-stored-ranged-response':
      useStoredRangeResponse(event);
      return;
    case 'broadcast-accept-encoding':
      broadcastAcceptEncoding(event);
      return;
    case 'record-media-range-request':
      return recordMediaRangeRequest(event);
    case 'use-media-range-request':
      useMediaRangeRequest(event);
      return;
  }
});

/**
 * @param {Request} request
 */
function rangeHeaderFilterTest(request) {
  const rangeValue = request.headers.get('Range');

  test(() => {
    assert_range_request(new Request(request), rangeValue, `Untampered`);
    assert_range_request(new Request(request, {}), rangeValue, `Untampered (no init props set)`);
    assert_range_request(new Request(request, { __foo: 'bar' }), rangeValue, `Untampered (only invalid props set)`);
    assert_range_request(new Request(request, { mode: 'cors' }), rangeValue, `More permissive mode`);
    assert_range_request(request.clone(), rangeValue, `Clone`);
  }, "Range headers correctly preserved");

  test(() => {
    assert_range_request(new Request(request, { headers: { Range: 'foo' } }), null, `Tampered - range header set`);
    assert_range_request(new Request(request, { headers: {} }), null, `Tampered - empty headers set`);
    assert_range_request(new Request(request, { mode: 'no-cors' }), null, `Tampered – mode set`);
    assert_range_request(new Request(request, { cache: 'no-cache' }), null, `Tampered – cache mode set`);
  }, "Range headers correctly removed");

  test(() => {
    let headers;

    headers = new Request(request).headers;
    headers.delete('does-not-exist');
    assert_equals(headers.get('Range'), rangeValue, `Preserved if no header actually removed`);

    headers = new Request(request).headers;
    headers.append('foo', 'bar');
    assert_equals(headers.get('Range'), rangeValue, `Preserved if silent-failure on append (due to request-no-cors guard)`);

    headers = new Request(request).headers;
    headers.set('foo', 'bar');
    assert_equals(headers.get('Range'), rangeValue, `Preserved if silent-failure on set (due to request-no-cors guard)`);

    headers = new Request(request).headers;
    headers.append('Range', 'foo');
    assert_equals(headers.get('Range'), rangeValue, `Preserved if silent-failure on append (due to request-no-cors guard)`);

    headers = new Request(request).headers;
    headers.set('Range', 'foo');
    assert_equals(headers.get('Range'), rangeValue, `Preserved if silent-failure on set (due to request-no-cors guard)`);

    headers = new Request(request).headers;
    headers.append('Accept', 'whatever');
    assert_equals(headers.get('Range'), null, `Stripped if header successfully appended`);

    headers = new Request(request).headers;
    headers.set('Accept', 'whatever');
    assert_equals(headers.get('Range'), null, `Stripped if header successfully set`);

    headers = new Request(request).headers;
    headers.delete('Accept');
    assert_equals(headers.get('Range'), null, `Stripped if header successfully deleted`);

    headers = new Request(request).headers;
    headers.delete('Range');
    assert_equals(headers.get('Range'), null, `Stripped if range header successfully deleted`);
  }, "Headers correctly filtered");

  done();
}

function rangeHeaderPassthroughTest(event) {
  /** @type Request */
  const request = event.request;
  const url = new URL(request.url);
  const key = url.searchParams.get('range-received-key');

  event.waitUntil(new Promise(resolve => {
    promise_test(async () => {
      await fetch(event.request);
      const response = await fetch('stash-take.py?key=' + key);
      assert_equals(await response.json(), 'range-header-received');
      resolve();
    }, `Include range header in network request`);

    done();
  }));

  // Just send back any response, it isn't important for the test.
  event.respondWith(new Response(''));
}

let storedRangeResponseP;

function storeRangedResponse(event) {
  /** @type Request */
  const request = event.request;
  const id = new URL(request.url).searchParams.get('id');

  storedRangeResponseP = fetch(event.request);
  broadcast({ id });

  // Just send back any response, it isn't important for the test.
  event.respondWith(new Response(''));
}

function useStoredRangeResponse(event) {
  event.respondWith(async function() {
    const response = await storedRangeResponseP;
    if (!response) throw Error("Expected stored range response");
    return response.clone();
  }());
}

function broadcastAcceptEncoding(event) {
  /** @type Request */
  const request = event.request;
  const id = new URL(request.url).searchParams.get('id');

  broadcast({
    id,
    acceptEncoding: request.headers.get('Accept-Encoding')
  });

  // Just send back any response, it isn't important for the test.
  event.respondWith(new Response(''));
}

let rangeResponse = {};

async function recordMediaRangeRequest(event) {
  /** @type Request */
  const request = event.request;
  const url = new URL(request.url);
  const urlParams = new URLSearchParams(url.search);
  const size = urlParams.get("size");
  const id = urlParams.get('id');
  const key = 'size' + size;

  if (key in rangeResponse) {
    // Don't re-fetch ranges we already have.
    const clonedResponse = rangeResponse[key].clone();
    event.respondWith(clonedResponse);
  } else if (event.request.headers.get("range") === "bytes=0-") {
    // Generate a bogus 206 response to trigger subsequent range requests
    // of the desired size.
    const length = urlParams.get("length") + 100;
    const body = "A".repeat(Number(size));
    event.respondWith(new Response(body, {status: 206, headers: {
      "Content-Type": "audio/mp4",
      "Content-Range": `bytes 0-1/${length}`
    }}));
  } else if (event.request.headers.get("range") === `bytes=${Number(size)}-`) {
    // Pass through actual range requests which will attempt to fetch up to the
    // length in the original response which is bigger than the actual resource
    // to make sure 206 and 416 responses are treated the same.
    rangeResponse[key] = await fetch(event.request);

    // Let the client know we have the range response for the given ID
    broadcast({id});
  } else {
    event.respondWith(Promise.reject(Error("Invalid Request")));
  }
}

function useMediaRangeRequest(event) {
  /** @type Request */
  const request = event.request;
  const url = new URL(request.url);
  const urlParams = new URLSearchParams(url.search);
  const size = urlParams.get("size");
  const key = 'size' + size;

  // Send a clone of the range response to preload.
  if (key in rangeResponse) {
    const clonedResponse = rangeResponse[key].clone();
    event.respondWith(clonedResponse);
  } else {
    event.respondWith(Promise.reject(Error("Invalid Request")));
  }
}
