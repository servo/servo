// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

const BAD_CERTIFICATE_HASH_OPTIONS = [
  { serverCertificateHashes: [{ value: new Uint8Array(32) }] },
  { serverCertificateHashes: [{ algorithm: "sha-256" }] },
  { serverCertificateHashes: [{}] },
];

for (const options of BAD_CERTIFICATE_HASH_OPTIONS) {
  test(() => {
    assert_throws_js(TypeError, () => new WebTransport(
                                         webtransport_url('echo.py'), options),
                     'constructor should throw');
  }, `WebTransport constructor should reject serverCertificateHashes ${JSON.stringify(options.serverCertificateHashes)}`);
}

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(webtransport_url(`client-close.py?token=${id}`),
                              { serverCertificateHashes: [] });
  await wt.ready;
  wt.close();
}, 'WebTransport constructor should allow an empty serverCertificateHashes list');

promise_test(async t => {
  const id = token();
  const wt = new WebTransport(
      webtransport_url(`client-close.py?token=${id}`),
      { allowPooling: true, serverCertificateHashes: [] });
  await wt.ready;
  wt.close();
}, 'WebTransport constructor should allow an empty serverCertificateHashes list with allowPooling');
