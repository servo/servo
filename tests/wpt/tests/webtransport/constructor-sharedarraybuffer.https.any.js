// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

// The value member of WebTransportHash is not [AllowShared], so a
// SharedArrayBuffer, or a view onto one, has to throw a TypeError.
//
// See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`.
const sharedBuffer = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 }).buffer;

for (const [kind, value] of [['SharedArrayBuffer', sharedBuffer],
                             ['Uint8Array(SharedArrayBuffer)', new Uint8Array(sharedBuffer)]]) {
  test(() => {
    assert_throws_js(TypeError, () => {
      const wt = new WebTransport(
          webtransport_url('echo.py'),
          { serverCertificateHashes: [{ algorithm: 'sha-256', value }] });
      // Only reached when the shared buffer was wrongly accepted. Close the
      // session so the test does not leave a connection attempt running.
      wt.ready.catch(() => {});
      wt.closed.catch(() => {});
      wt.close();
    });
  }, `WebTransport constructor should reject serverCertificateHashes value of type '${kind}'`);
}
