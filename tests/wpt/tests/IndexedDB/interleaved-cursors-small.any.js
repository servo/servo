// META: title=IndexedDB: Interleaved iteration of multiple cursors
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/interleaved-cursors-common.js
// META: timeout=long

'use strict';

cursorTest(1);
cursorTest(10);
cursorTest(100);
