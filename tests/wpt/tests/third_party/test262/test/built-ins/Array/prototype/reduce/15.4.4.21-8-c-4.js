// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce doesn't throw error when array has no own
    properties but prototype contains a single property
---*/

var arr = [, , , ];

try {
  Array.prototype[1] = "prototype";
  arr.reduce(function() {});
} finally {
  delete Array.prototype[1];
}
