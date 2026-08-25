// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  The `start` index parameter is converted to an integral numeric value.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  6. Let relativeStart be ToInteger(start).
  7. ReturnIfAbrupt(relativeStart).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var start = 4.5,
  end = 8;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 4, "slice(4.5, 8)");

var start = NaN,
  end = 8;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 8, "slice(NaN, 8)");
