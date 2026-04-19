// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Negative `start` index is relative to [[ArrayBufferByteLength]].
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);

var start = -5,
  end = 6;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 3, "slice(-5, 6)");

var start = -12,
  end = 6;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 6, "slice(-12, 6)");

var start = -Infinity,
  end = 6;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 6, "slice(-Infinity, 6)");
