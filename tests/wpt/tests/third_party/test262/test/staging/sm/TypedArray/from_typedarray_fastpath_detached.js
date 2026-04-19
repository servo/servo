// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
// Ensure the fast-path when TypedArray.from is called with a TypedArray still
// checks for detached buffers.

var ta = new Int32Array(4);
$DETACHBUFFER(ta.buffer);

assert.throws(TypeError, () => Int32Array.from(ta));

