// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
let ta = new BigInt64Array(10);

let obj = {
  get length() {
    $DETACHBUFFER(ta.buffer);
    return 1;
  },
  0: {
    valueOf() {
      return "huzzah!";
    }
  },
};

// Throws a SyntaxError, because "huzzah!" can't be parsed as a BigInt.
assert.throws(SyntaxError, () => ta.set(obj));

