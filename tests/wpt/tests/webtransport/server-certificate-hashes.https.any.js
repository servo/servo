// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const hashValue = new Uint8Array(32);
  // The connection fails because the fingerprint does not match.
  const wt = new WebTransport(webtransport_url('echo.py'), {
    serverCertificateHashes: [
      {
        algorithm: "sha-256",
        value: hashValue,
      }
    ]
  });

  const e = await wt.ready.catch(e => e);
  await promise_rejects_exactly(t, e, wt.ready, 'ready should be rejected');
  await promise_rejects_exactly(t, e, wt.closed, 'closed should be rejected');
  assert_true(e instanceof WebTransportError);
}, 'Connection fails due to certificate hash mismatch');
