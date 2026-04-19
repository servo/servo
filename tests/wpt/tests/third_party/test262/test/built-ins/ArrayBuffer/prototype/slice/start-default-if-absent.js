// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  The `start` index defaults to 0 if absent.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  6. Let relativeStart be ToInteger(start).
  7. ReturnIfAbrupt(relativeStart).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var result = arrayBuffer.slice();
assert.sameValue(result.byteLength, 8);
