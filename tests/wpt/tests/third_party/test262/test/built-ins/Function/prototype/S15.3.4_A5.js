// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Function prototype object is itself a Function object without
    [[Construct]] property
es5id: 15.3.4_A5
description: Checking if creating "new Function.prototype object" fails
---*/

assert.throws(TypeError, function() {
  new Function.prototype;
}, '`new Function.prototype` throws a TypeError exception');
