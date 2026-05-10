// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
es5id: 15.4.4.18-4-6
description: Array.prototype.forEach throws TypeError if callbackfn is string
---*/

var arr = new Array(10);
assert.throws(TypeError, function() {
  arr.forEach("abc");
});
