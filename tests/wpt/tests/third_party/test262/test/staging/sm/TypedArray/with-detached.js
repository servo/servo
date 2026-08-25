// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

var ta = new Int32Array([3, 2, 1]);

$DETACHBUFFER(ta.buffer);

assert.throws(TypeError, () => ta.with(0, 0));

