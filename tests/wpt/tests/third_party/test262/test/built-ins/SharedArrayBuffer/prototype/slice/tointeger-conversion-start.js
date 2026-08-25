// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The `start` index parameter is converted to an integral numeric value.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )
features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);
var result;

result = arrayBuffer.slice(4.5, 8);
assert.sameValue(result.byteLength, 4, "slice(4.5, 8)");

result = arrayBuffer.slice(NaN, 8);
assert.sameValue(result.byteLength, 8, "slice(NaN, 8)");
