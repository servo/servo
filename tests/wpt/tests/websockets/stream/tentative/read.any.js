// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: script=/common/gc.js
// META: global=window,worker
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

'use strict';

promise_test(async t => {
    const wss = new WebSocketStream(ECHOURL);
    const { readable, writable } = await wss.opened;
    const writer = writable.getWriter();
    const MESSAGE_SIZE = 16;
    writer.write(new Uint8Array(MESSAGE_SIZE));
    const reader = readable.getReader();
    const { value, done } = await reader.read();
    assert_false(done, 'done should be false');
    assert_true(value instanceof Uint8Array,
        'instanceof should give Uint8Array');
    assert_equals(value.constructor, Uint8Array,
        'value should be a Uint8Array');
    assert_true(ArrayBuffer.isView(value),
        'value should be a view');
    assert_equals(value.length, MESSAGE_SIZE, 'length should match');
    assert_equals(value.byteLength, MESSAGE_SIZE,
        'binary_length should match');
}, 'read type for binary messages is Uint8Array');
