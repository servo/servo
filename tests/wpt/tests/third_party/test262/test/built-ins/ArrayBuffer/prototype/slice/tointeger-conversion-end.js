// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  The `end` index parameter is converted to an integral numeric value.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  9. If end is undefined, let relativeEnd be len; else let relativeEnd be ToInteger(end).
  10. ReturnIfAbrupt(relativeEnd).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var start = 0,
  end = 4.5;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 4, "slice(0, 4.5)");

var start = 0,
  end = NaN;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0, "slice(0, NaN)");
