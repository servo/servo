// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf returns -1 if 'length' is 0 (generic
    'array' with length 0 )
---*/

var i = Array.prototype.indexOf.call({
  length: 0
}, 1);


assert.sameValue(i, -1, 'i');
