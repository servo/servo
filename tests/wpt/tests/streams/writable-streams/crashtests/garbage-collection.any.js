// META: global=window,worker
// META: script=/common/gc.js
'use strict';

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
  let writer = new WritableStream({
    write(chunk) { return new Promise(resolve => {}); },
    close() { return new Promise(resolve => {}); }
  }).getWriter();
  writer.write('is there anyone home?');
  writer.close();
  writer = null;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream writer should not crash with close promise pending');

promise_test(async () => {
  const ready = new WritableStream({
    write(chunk) { }
  }, {highWaterMark: 0}).getWriter().ready;
  for (let i = 0; i < 5; ++i)
    await garbageCollect();
}, 'Garbage-collecting a stream writer should not crash when backpressure is being applied');

