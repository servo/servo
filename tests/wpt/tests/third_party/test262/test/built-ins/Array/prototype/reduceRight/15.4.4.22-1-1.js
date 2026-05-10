// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight applied to undefined throws a TypeError
---*/


assert.throws(TypeError, function() {
  Array.prototype.reduceRight.call(undefined);
});
