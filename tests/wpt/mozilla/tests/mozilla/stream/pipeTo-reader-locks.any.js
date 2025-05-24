'use strict';

promise_test(t => {
    const rs = new ReadableStream();
    const ws = new WritableStream();

    rs.getReader();

    assert_true(rs.locked, 'sanity check: the ReadableStream starts locked');
    assert_false(ws.locked, 'sanity check: the WritableStream does not start locked');

    return promise_rejects_js(t, TypeError, rs.pipeTo(ws)).then(() => {
        assert_false(ws.locked, 'the WritableStream must still be unlocked');
    });

}, 'pipeTo must fail if the ReadableStream is locked with a Default reader, and not lock the WritableStream');


promise_test(t => {
    const rs = new ReadableStream({
        type: 'bytes',
        pull(controller) {
            // No chunks are enqueued; the stream remains readable but empty.
        }
    });
    const ws = new WritableStream();
    rs.getReader({ mode: 'byob' });
    assert_true(rs.locked, 'sanity check: the ReadableStream is locked by the BYOB reader');
    assert_false(ws.locked, 'sanity check: the WritableStream starts unlocked');
    return promise_rejects_js(t, TypeError, rs.pipeTo(ws)).then(() => {
        assert_false(ws.locked, 'the WritableStream must remain unlocked after pipeTo rejection');
    });
}, 'pipeTo must fail if the ReadableStream is locked with a BYOB reader, and must not lock the WritableStream');
