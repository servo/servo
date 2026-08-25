// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.4_A14
description: If IsCallable(func) is false, then throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  Function.prototype.call.call(null, {});
}, 'Function.prototype.call.call(null, {}) throws a TypeError exception');
