// META: timeout=long
// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

// Helpers that return headers objects with a particular guard
function headersGuardNone(fill) {
  if (fill) return new Headers(fill);
  return new Headers();
}

function headersGuardResponse(fill) {
  const opts = {};
  if (fill) opts.headers = fill;
  return new Response('', opts).headers;
}

function headersGuardRequest(fill) {
  const opts = {};
  if (fill) opts.headers = fill;
  return new Request('./', opts).headers;
}

function headersGuardRequestNoCors(fill) {
  const opts = { mode: 'no-cors' };
  if (fill) opts.headers = fill;
  return new Request('./', opts).headers;
}

const headerGuardTypes = [
  ['none', headersGuardNone],
  ['response', headersGuardResponse],
  ['request', headersGuardRequest]
];

for (const [guardType, createHeaders] of headerGuardTypes) {
  test(() => {
    // There are three ways to set headers.
    // Filling, appending, and setting. Test each:
    let headers = createHeaders({ Range: 'foo' });
    assert_equals(headers.get('Range'), 'foo');

    headers = createHeaders();
    headers.append('Range', 'foo');
    assert_equals(headers.get('Range'), 'foo');

    headers = createHeaders();
    headers.set('Range', 'foo');
    assert_equals(headers.get('Range'), 'foo');
  }, `Range header setting allowed for guard type: ${guardType}`);
}

test(() => {
  let headers = headersGuardRequestNoCors({ Range: 'foo' });
  assert_false(headers.has('Range'));

  headers = headersGuardRequestNoCors();
  headers.append('Range', 'foo');
  assert_false(headers.has('Range'));

  headers = headersGuardRequestNoCors();
  headers.set('Range', 'foo');
  assert_false(headers.has('Range'));
}, `Privileged header not allowed for guard type: request-no-cors`);

promise_test(async () => {
  const wavURL = new URL('resources/long-wav.py', location);
  const stashTakeURL = new URL('resources/stash-take.py', location);

  function changeToken() {
    const stashToken = token();
    wavURL.searchParams.set('accept-encoding-key', stashToken);
    stashTakeURL.searchParams.set('key', stashToken);
  }

  const rangeHeaders = [
    'bytes=0-10',
    'foo=0-10',
    'foo',
    ''
  ];

  for (const rangeHeader of rangeHeaders) {
    changeToken();

    await fetch(wavURL, {
      headers: { Range: rangeHeader }
    });

    const response = await fetch(stashTakeURL);

    assert_regexp_match(await response.json(),
                        /.*\bidentity\b.*/,
                        `Expect identity accept-encoding if range header is ${JSON.stringify(rangeHeader)}`);
  }
}, `Fetch with range header will be sent with Accept-Encoding: identity`);

promise_test(async () => {
  const wavURL = new URL(get_host_info().HTTP_REMOTE_ORIGIN + '/fetch/range/resources/long-wav.py');
  const stashTakeURL = new URL('resources/stash-take.py', location);

  function changeToken() {
    const stashToken = token();
    wavURL.searchParams.set('accept-encoding-key', stashToken);
    stashTakeURL.searchParams.set('key', stashToken);
  }

  const rangeHeaders = [
    'bytes=10-9',
    'bytes=-0',
    'bytes=0000000000000000000000000000000000000000000000000000000000011-0000000000000000000000000000000000000000000000000000000000111',
  ];

  for (const rangeHeader of rangeHeaders) {
    changeToken();
    await fetch(wavURL, { headers: { Range : rangeHeader} }).then(() => { throw "loaded with range header " + rangeHeader }, () => { });
  }
}, `Cross Origin Fetch with non safe range header`);

promise_test(async () => {
  const wavURL = new URL(get_host_info().HTTP_REMOTE_ORIGIN + '/fetch/range/resources/long-wav.py');
  const stashTakeURL = new URL('resources/stash-take.py', location);

  function changeToken() {
    const stashToken = token();
    wavURL.searchParams.set('accept-encoding-key', stashToken);
    stashTakeURL.searchParams.set('key', stashToken);
  }

  const rangeHeaders = [
    'bytes=0-10',
    'bytes=0-',
    'bytes=00000000000000000000000000000000000000000000000000000000011-00000000000000000000000000000000000000000000000000000000000111',
  ];

  for (const rangeHeader of rangeHeaders) {
    changeToken();
    await fetch(wavURL, { headers: { Range: rangeHeader } }).then(() => { }, () => { throw "failed load with range header " + rangeHeader });
  }
}, `Cross Origin Fetch with safe range header`);
