// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce throws TypeError if 'length' is 0 (empty
    array), no initVal
---*/

function cb() {}
assert.throws(TypeError, function() {
  [].reduce(cb);
});
