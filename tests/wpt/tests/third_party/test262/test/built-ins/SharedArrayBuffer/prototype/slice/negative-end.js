// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Negative `end` index is relative to [[ArrayBufferByteLength]].
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);

var start = 2,
  end = -4;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 2, "slice(2, -4)");

var start = 2,
  end = -10;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(2, -10)");

var start = 2,
  end = -Infinity;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(2, -Infinity)");
