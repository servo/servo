// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The `start` index defaults to 0 if absent.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer]
---*/

var arrayBuffer = new SharedArrayBuffer(8);

var result = arrayBuffer.slice();
assert.sameValue(result.byteLength, 8);
