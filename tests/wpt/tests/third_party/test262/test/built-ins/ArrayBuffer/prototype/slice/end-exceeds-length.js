// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Large `end` index is clamped to [[ArrayBufferByteLength]].
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  8. If relativeEnd < 0, let final be max((len + relativeEnd),0); else let final be min(relativeEnd, len).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

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
