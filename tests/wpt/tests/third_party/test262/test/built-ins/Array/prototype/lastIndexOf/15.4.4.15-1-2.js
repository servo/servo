// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf applied to null throws a TypeError
---*/


assert.throws(TypeError, function() {
  Array.prototype.lastIndexOf.call(null);
});
