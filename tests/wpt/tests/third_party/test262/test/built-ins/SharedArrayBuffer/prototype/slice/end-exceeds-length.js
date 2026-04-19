// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Large `end` index is clamped to [[ArrayBufferByteLength]].
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);

var start = 1,
  end = 12;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 7, "slice(1, 12)");

var start = 2,
  end = 0x100000000;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 6, "slice(2, 0x100000000)");

var start = 3,
  end = +Infinity;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 5, "slice(3, Infinity)");
