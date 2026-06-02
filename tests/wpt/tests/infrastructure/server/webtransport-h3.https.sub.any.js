// META: global=window,worker
// META: script=/common/get-host-info.sub.js

const HOST = get_host_info().ORIGINAL_HOST;
const PORT = '{{ports[webtransport-h3][0]}}';
const BASE = `https://${HOST}:${PORT}`;

promise_test(async t => {
    const wt = new WebTransport(`${BASE}/webtransport/handlers/echo.py`);
    // When a connection fails `closed` attribute will be rejected.
    wt.closed.catch((error) => {
        t.unreached_func(`The 'closed' attribute should not be rejected: ${error}`);
    });
    await wt.ready;

    const stream = await wt.createBidirectionalStream();

    const writer = stream.writable.getWriter();
    await writer.write(new Uint8Array([42]));
    writer.releaseLock();

    const reader = stream.readable.getReader();
    const { value } = await reader.read();

    assert_equals(value[0], 42);
}, "WebTransport server should be running and should handle a bidirectional stream");
