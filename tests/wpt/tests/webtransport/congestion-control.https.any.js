// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

// Tests that a WebTransport session can be established when congestionControl
// is specified in the constructor, and that the attribute is readable.

const CONGESTION_CONTROL_VALUES = ["default", "throughput", "low-latency"];

for (const value of CONGESTION_CONTROL_VALUES) {
  promise_test(async t => {
    const id = token();
    const wt = new WebTransport(
        webtransport_url(`client-close.py?token=${id}`),
        {congestionControl: value});
    await wt.ready;

    // The congestionControl attribute should be readable after connection.
    // The spec says if the UA doesn't support the requested algorithm, it
    // falls back to "default". So the value is either what was requested or
    // "default".
    assert_in_array(wt.congestionControl, [value, "default"],
        'congestionControl should be the requested value or "default"');

    wt.close();
  }, `WebTransport session established with congestionControl "${value}" and attribute is readable`);
}

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(
      webtransport_url(`client-close.py?token=${id}`));
  await wt.ready;

  // When no congestionControl is specified, the default should be used.
  assert_equals(wt.congestionControl, "default",
      'congestionControl should default to "default"');

  wt.close();
}, 'WebTransport session without congestionControl option defaults to "default"');
