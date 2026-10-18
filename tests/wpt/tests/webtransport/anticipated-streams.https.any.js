// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, null,
                'initial uni value should be null');
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, null,
                'initial bidi value should be null');

  wt.anticipatedConcurrentIncomingUnidirectionalStreams = 65535;
  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, 65535,
                'uni value should be set');

  wt.anticipatedConcurrentIncomingBidirectionalStreams = 65535;
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, 65535,
                'bidi value should be set');

  wt.anticipatedConcurrentIncomingUnidirectionalStreams = null;
  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, null,
                'uni value should be nullable');

  wt.anticipatedConcurrentIncomingBidirectionalStreams = null;
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, null,
                'bidi value should be nullable');
}, 'anticipatedConcurrentIncoming* attributes should be mutable');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  wt.anticipatedConcurrentIncomingUnidirectionalStreams = 0;
  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, 0,
                'uni value should accept 0');

  wt.anticipatedConcurrentIncomingBidirectionalStreams = 0;
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, 0,
                'bidi value should accept 0');
}, 'anticipatedConcurrentIncoming* attributes should accept 0');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  t.add_cleanup(() => wt.close());

  assert_throws_js(TypeError, () => wt.anticipatedConcurrentIncomingUnidirectionalStreams = -1,
                   'should throw TypeError on negative anticipated streams');
  assert_throws_js(TypeError, () => wt.anticipatedConcurrentIncomingBidirectionalStreams = -1,
                   'should throw TypeError on netative anticipated streams');
  assert_throws_js(TypeError, () => wt.anticipatedConcurrentIncomingUnidirectionalStreams = 65536,
                   'should throw TypeError on max+1 anticipated streams');
  assert_throws_js(TypeError, () => wt.anticipatedConcurrentIncomingBidirectionalStreams = 65536,
                   'should throw TypeError on max+1 anticipated streams');
}, 'anticipatedConcurrentIncoming* attributes should throw on invalid u16 values');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  wt.anticipatedConcurrentIncomingUnidirectionalStreams = 100;
  wt.anticipatedConcurrentIncomingBidirectionalStreams = 50;

  wt.close();
  await wt.closed;

  assert_equals(wt.anticipatedConcurrentIncomingUnidirectionalStreams, 100,
                'values should persist after close');
  assert_equals(wt.anticipatedConcurrentIncomingBidirectionalStreams, 50,
                'values should persist after close');
}, 'anticipatedConcurrentIncoming* attributes should persist after session close');

async function first(readable) {
  for await (const v of readable.values({ preventCancel: true })) return v;
}

// Helper function to request stream creation and get the response
async function requestServerStreamLimits(wt, numUni, numBidi) {
  const { readable, writable } = await first(wt.incomingBidirectionalStreams);

  const writer = writable.getWriter();
  const request = JSON.stringify({ uni: numUni, bidi: numBidi });
  writer.write(new TextEncoder().encode(request)).catch(() => {});
  await writer.close();

  let responseText = "";
  for await (const txt of readable.pipeThrough(new TextDecoderStream())) {
    responseText += txt;
  }

  return JSON.parse(responseText);
}

promise_test(async t => {
  assert_throws_js(TypeError, () => {
    new WebTransport(webtransport_url('anticipated-streams.py'), {
      anticipatedConcurrentIncomingUnidirectionalStreams: 65536,
    });
  });

  assert_throws_js(TypeError, () => {
    new WebTransport(webtransport_url('anticipated-streams.py'), {
      anticipatedConcurrentIncomingUnidirectionalStreams: -1,
    });
  });
}, 'WebTransportOptions enforces limits on min/maximum anticipated streams');

// Note: the values here need to be larger than the normally-negotiated initial values
promise_test(async t => {
  const wt = new WebTransport(webtransport_url('anticipated-streams.py'), {
    anticipatedConcurrentIncomingUnidirectionalStreams: 500,
    anticipatedConcurrentIncomingBidirectionalStreams: 300
  });
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const result = await requestServerStreamLimits(wt, 500, 300);

  assert_greater_than_equal(result.uni_created, 500,
                            'server should create at least 500 unidirectional streams');
  assert_greater_than_equal(result.bidi_created, 300,
                            'server should create at least 300 bidirectional streams');
}, 'server can create anticipated number of unidirectional and bidirectional streams');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('anticipated-streams.py'), {
    anticipatedConcurrentIncomingUnidirectionalStreams: 1000
  });
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const result = await requestServerStreamLimits(wt, 1000, 0);

  assert_greater_than_equal(result.uni_created, 1000,
                            'server should create at least 1000 unidirectional streams');
}, 'server can create 1000 anticipated unidirectional streams');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('anticipated-streams.py'), {
    anticipatedConcurrentIncomingBidirectionalStreams: 2000
  });
  await wt.ready;
  t.add_cleanup(() => wt.close());

  const result = await requestServerStreamLimits(wt, 0, 2000);

  assert_greater_than_equal(result.bidi_created, 2000,
                            'server should create at least 2000 bidirectional streams');
}, 'server can create 2000 anticipated bidirectional streams');
