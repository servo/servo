// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Returns zero-length buffer if `start` index exceeds `end` index.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  12. Let newLen be max(final-first,0).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var start = 5,
  end = 4;
var result = arrayBuffer.slice(start, end);
assert.sameValue(result.byteLength, 0);
