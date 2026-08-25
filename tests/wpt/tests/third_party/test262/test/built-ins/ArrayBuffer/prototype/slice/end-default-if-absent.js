// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  The `end` index defaults to [[ArrayBufferByteLength]] if absent.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  9. If end is undefined, let relativeEnd be len; else let relativeEnd be ToInteger(end).
  10. ReturnIfAbrupt(relativeEnd).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var start = 6;
var result = arrayBuffer.slice(start);
assert.sameValue(result.byteLength, 2);
