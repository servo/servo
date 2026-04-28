// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight throws TypeError if 'length' is 0
    (empty array), no initVal
---*/

function cb() {}
assert.throws(TypeError, function() {
  [].reduceRight(cb);
});
