// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Negative `end` index is relative to [[ArrayBufferByteLength]].
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  8. If relativeEnd < 0, let final be max((len + relativeEnd),0); else let final be min(relativeEnd, len).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

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
