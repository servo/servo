// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Large `start` index is clamped to [[ArrayBufferByteLength]].
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  8. If relativeStart < 0, let first be max((len + relativeStart),0); else let first be min(relativeStart, len).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var start = 10,
  end = 8;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(10, 8)");

var start = 0x100000000,
  end = 7;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(0x100000000, 7)");

var start = +Infinity,
  end = 6;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(+Infinity, 6)");
