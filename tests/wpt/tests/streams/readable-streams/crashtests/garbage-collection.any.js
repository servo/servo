// META: global=window,worker
// META: script=/common/gc.js
'use strict';

// See https://crbug.com/335506658 for details.
promise_test(async () => {
  const closed = new ReadableStream({
      pull(controller) {
        controller.enqueue('is there anybody in there?');
      }
  }).getReader().closed;
  // 3 GCs are actually required to trigger the bug at time of writing.
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream along with its reader should not crash');


// See https://crbug.com/390646657 for details.
promise_test(async () => {
  const written = new WritableStream({
    write(chunk) {
      return new Promise(resolve => {});
    }
  }).getWriter().write('just nod if you can hear me');
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream writer with a pending write should not crash');


promise_test(async () => {
  const closed = new WritableStream({
    write(chunk) { }
  }).getWriter().closed;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream writer should not crash with closed promise is retained');

promise_test(async () => {
  const ready = new WritableStream({
    write(chunk) { }
  }, {highWaterMark: 0}).getWriter().ready;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream writer should not crash when backpressure is being applied');

promise_test(async () => {
  let reader = new ReadableStream({
      pull() { }
  }).getReader();
  const promise = reader.read();
  reader = null;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream with a pending read should not crash');

promise_test(async () => {
  let reader = new ReadableStream({
      type: "bytes",
      pull() { return new Promise(resolve => {}); }
  }).getReader({mode: "byob"});
  const promise = reader.read(new Uint8Array(42));
  reader = null;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream with a pending BYOB read should not crash');


