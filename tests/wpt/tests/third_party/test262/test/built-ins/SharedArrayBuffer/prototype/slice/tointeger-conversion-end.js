// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The `end` index parameter is converted to an integral numeric value.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )
features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);
var result;

result = arrayBuffer.slice(0, 4.5);
assert.sameValue(result.byteLength, 4, "slice(0, 4.5)");

result = arrayBuffer.slice(0, NaN);
assert.sameValue(result.byteLength, 0, "slice(0, NaN)");
