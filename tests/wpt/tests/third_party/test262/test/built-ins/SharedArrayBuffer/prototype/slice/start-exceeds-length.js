// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Large `start` index is clamped to [[ArrayBufferByteLength]].
info: |
  SharedArrayBuffer.prototype.slice ( start, end )
features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);
var result;

result = arrayBuffer.slice(10, 8);
assert.sameValue(result.byteLength, 0, "slice(10, 8)");

result = arrayBuffer.slice(0x100000000, 7);
assert.sameValue(result.byteLength, 0, "slice(0x100000000, 7)");

result = arrayBuffer.slice(+Infinity, 6);
assert.sameValue(result.byteLength, 0, "slice(+Infinity, 6)");
