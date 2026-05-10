// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to null throws a TypeError
---*/


assert.throws(TypeError, function() {
  Array.prototype.every.call(null); // TypeError is thrown if value is null
});
