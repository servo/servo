// META: global=window,worker
// META: script=/common/gc.js
'use strict';

// See https://crbug.com/335506658 for details.
promise_test(async () => {
    let closed = new ReadableStream({
        pull(controller) {
          controller.enqueue('is there anybody in there?');
        }
    }).getReader().closed;
    // 3 GCs are actually required to trigger the bug at time of writing.
    for (let i = 0; i < 5; ++i)
      await garbageCollect();
}, 'Garbage-collecting a stream along with its reader should not crash');
