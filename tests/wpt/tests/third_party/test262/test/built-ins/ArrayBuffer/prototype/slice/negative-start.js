// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Negative `start` index is relative to [[ArrayBufferByteLength]].
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  8. If relativeStart < 0, let first be max((len + relativeStart),0); else let first be min(relativeStart, len).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

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
