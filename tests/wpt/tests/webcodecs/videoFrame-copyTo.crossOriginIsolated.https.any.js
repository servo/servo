// META: global=window,dedicatedworker
// META: script=/webcodecs/videoFrame-utils.js

promise_test(async t => {
  // *.headers file should ensure we sesrve COOP and COEP headers.
  assert_true(self.crossOriginIsolated,
    "Cross origin isolation is required to construct SharedArrayBuffer");
  const destination = new SharedArrayBuffer(I420_DATA.length);
  await testI420_4x2_copyTo(destination);
}, 'Test copying I420 frame to SharedArrayBuffer.');

promise_test(async t => {
  // *.headers file should ensure we sesrve COOP and COEP headers.
  assert_true(self.crossOriginIsolated,
    "Cross origin isolation is required to construct SharedArrayBuffer");
  const destination = new Uint8Array(new SharedArrayBuffer(I420_DATA.length));
  await testI420_4x2_copyTo(destination);
}, 'Test copying I420 frame to shared ArrayBufferView.');
