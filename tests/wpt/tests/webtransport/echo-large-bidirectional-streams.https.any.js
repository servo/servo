// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

// A test that aims to reproduce https://crbug.com/1369030 -- note that since
// the bug in question is a race condition, this test will probably be flaky if
// this is actually broken.
promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const numBytes = 1024 * 1024;
  const numStreams = 5;
  for (let i = 0; i < numStreams; i++) {
    const stream = await wt.createBidirectionalStream();
    const writer = stream.writable.getWriter();
    await writer.write(new Uint8Array(numBytes));
    await writer.close();
    const response = await (new Response(stream.readable).arrayBuffer());
    assert_equals(response.byteLength, numBytes);
  }
}, 'Ensure large bidirectional streams does not cause race condition');
