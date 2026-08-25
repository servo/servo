// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to undefined
---*/


assert.throws(TypeError, function() {
  Array.prototype.forEach.call(undefined); // TypeError is thrown if value is undefined
});
