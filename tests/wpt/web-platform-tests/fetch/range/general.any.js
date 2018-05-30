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

