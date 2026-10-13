// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

async function createWebTransport(t, options = {}) {
  const wt = new WebTransport(webtransport_url('echo.py'), options);
  t.add_cleanup(() => wt.close());
  wt.closed.catch(() => {});
  await wt.ready;
  return wt;
}

const NULL_HINT_CASES = [
  {description: 'omitted', options: {}},
  {
    description: 'explicitly null',
    options: {
      anticipatedConcurrentIncomingUnidirectionalStreams: null,
      anticipatedConcurrentIncomingBidirectionalStreams: null,
    },
  },
];

for (const {description, options} of NULL_HINT_CASES) {
  promise_test(async t => {
    const wt = await createWebTransport(t, options);

    assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, null,
                  'unidirectional hint should be null');
    assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, null,
                  'bidirectional hint should be null');
  }, `Anticipated incoming stream hints are null when ${description}`);
}

const HINT_NAMES = [
  'anticipatedConcurrentIncomingUnidirectionalStreams',
  'anticipatedConcurrentIncomingBidirectionalStreams',
];

const CONSTRUCTOR_HINT_CASES = [
  [99, 101],
  [0, 65535],
  [65535, 0],
];

for (const [unidirectional, bidirectional] of CONSTRUCTOR_HINT_CASES) {
  promise_test(async t => {
    const wt = await createWebTransport(t, {
      anticipatedConcurrentIncomingUnidirectionalStreams: unidirectional,
      anticipatedConcurrentIncomingBidirectionalStreams: bidirectional,
    });

    assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams,
                  unidirectional);
    assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams,
                  bidirectional);
  }, `Constructor reflects stream hints ${unidirectional}/${bidirectional}`);
}

promise_test(async t => {
  const wt = await createWebTransport(t);

  for (const boundary of [0, 65535]) {
    for (const name of HINT_NAMES) {
      wt[name] = boundary;
      assert_equals(wt[name], boundary,
                    `${name} setter should accept ${boundary}`);
    }
  }

  wt.anticipatedConcurrentIncomingUnidirectionalStreams = null;
  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, null,
                'unidirectional setter should accept null');
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, 65535,
                'clearing unidirectional should not affect bidirectional');
  wt.anticipatedConcurrentIncomingBidirectionalStreams = null;
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, null,
                'bidirectional setter should accept null');
}, 'Anticipated incoming stream hint setters accept boundary and null values');

const OUT_OF_RANGE_VALUES = [-1, 65536, Infinity, -Infinity, NaN];

for (const name of HINT_NAMES) {
  for (const value of OUT_OF_RANGE_VALUES) {
    test(t => {
      assert_throws_js(TypeError, () => {
        const wt =
            new WebTransport(webtransport_url('echo.py'), {[name]: value});
        t.add_cleanup(() => wt.close());
        wt.ready.catch(() => {});
        wt.closed.catch(() => {});
      });
    }, `WebTransport constructor rejects ${name} value ${value}`);
  }
}

for (const name of HINT_NAMES) {
  promise_test(async t => {
    const wt = await createWebTransport(t, {[name]: 42});

    for (const value of OUT_OF_RANGE_VALUES) {
      assert_throws_js(TypeError, () => wt[name] = value);
      assert_equals(wt[name], 42, 'rejected assignment should not update');
    }
  }, `${name} setter rejects out-of-range values`);
}
