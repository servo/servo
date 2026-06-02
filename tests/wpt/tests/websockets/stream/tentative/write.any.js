// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: script=/common/gc.js
// META: global=window,worker
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

'use strict';

const GOODBYE_MESSAGE = 'Goodbye';  // Must match echo_exit_wsh.py

// This message needs to be large enough that writing it cannot complete
// synchronously, and to fill up the TCP send buffer and any user agent internal
// send buffers so that the user agent has to receive the "Close" frame from the
// server before it can complete sending this message.
const BIG_MESSAGE_SIZE = 8 * 1024 * 1024;

// Common setup used by two tests. Sends a "Goodbye" message to tell the server
// to close the WebSocket, and immediately afterwards a big message that cannot
// be completely sent before the connection closes. Waits for the "Goodbye"
// message to be sent and the connection to be closed before returning. `t` is
// the test object provided by promse_test.
async function sendGoodbyeThenBigMessage(t) {
  const wss = new WebSocketStream(BASEURL + '/echo_exit');
  const { writable } = await wss.opened;
  const writer = writable.getWriter();
  const bigMessage = new Uint8Array(BIG_MESSAGE_SIZE);
  const goodbyePromise = writer.write(GOODBYE_MESSAGE);
  const bigMessagePromise = writer.write(bigMessage);
  await goodbyePromise;
  // testharness.js doesn't know about WebSocketError yet.
  await wss.closed.then(
      t.unreached_func('closed promise should reject'),
      e => assert_equals(
          e.constructor, WebSocketError,
          'a WebSocketError should be thrown'));
  return { writer, bigMessagePromise };
}

promise_test(async t => {
  const { writer, bigMessagePromise } = await sendGoodbyeThenBigMessage(t);
  await promise_rejects_dom(
      t, 'InvalidStateError', bigMessagePromise,
      'write() should reject with an InvalidStateError');
  const invalidStateError = await bigMessagePromise.then(
      t.unreached_func('write() promise should reject'), e => e);
  await promise_rejects_exactly(
      t, invalidStateError, writer.write('word'),
      'stream should be errored with same object');
}, 'a write that was incomplete at close time should reject');

promise_test(async t => {
  const { bigMessagePromise } = await sendGoodbyeThenBigMessage(t);
  // For some reason 5 is the magic number that causes garbage collection to
  // really really collect garbage.
  for (let i = 0; i < 5; ++i) {
    await garbageCollect();
  }
  await promise_rejects_dom(
      t, 'InvalidStateError', bigMessagePromise,
      'write() should reject with an InvalidStateError');
}, 'garbage collection after close with a pending write promise should not ' +
       'crash');

promise_test(async t => {
  const wss = new WebSocketStream(ECHOURL);
  const { writable } = await wss.opened;
  const writer = writable.getWriter();
  const cannotStringify = { toString() { return this; } };
  await promise_rejects_js(
      t, TypeError, writer.write(cannotStringify), 'write() should reject');
}, 'writing a value that cannot be stringified should cause a rejection');

promise_test(async t => {
  const wss = new WebSocketStream(ECHOURL);
  const { writable } = await wss.opened;
  const writer = writable.getWriter();
  const buffer = new ArrayBuffer(1024, { maxByteLength: 65536 });
  await promise_rejects_js(
      t, TypeError, writer.write(buffer), 'write() should reject');
}, 'writing a resizable ArrayBuffer should be rejected');

promise_test(async t => {
  const wss = new WebSocketStream(ECHOURL);
  const { writable } = await wss.opened;
  const writer = writable.getWriter();
  const memory = new WebAssembly.Memory({
    initial: 4096,
    maximum: 65536,
    shared: true,
  });
  const view = new Uint8Array(memory.buffer);
  await promise_rejects_js(
      t, TypeError, writer.write(view), 'write() should reject');
}, 'writing a view on a shared buffer should be rejected');

promise_test(async () => {
  let wss = new WebSocketStream(ECHOURL);
  let { writable } = await wss.opened;
  let writer = writable.getWriter();
  wss = writable = null;
  const promises = [];
  for (let i = 0; i < 20; ++i) {
    promises.push(writer.write(new Uint8Array(100000)));
  }
  writer = null;
  for (let i = 0; i < 5; ++i) {
    await garbageCollect();
  }
}, 'Garbage collecting a WebSocket stream doesn\'t crash while write promise is pending');
