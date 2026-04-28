// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Let O be the result of calling ToObject passing the this value as the
    argument.
es5id: 15.2.4.4_A15
description: Checking Object.prototype.valueOf when called as a global function.
---*/

assert.throws(TypeError, function() {
  const valueOf = Object.prototype.valueOf;
  valueOf();
}, '`const valueOf = Object.prototype.valueOf; valueOf()` throws a TypeError exception');
