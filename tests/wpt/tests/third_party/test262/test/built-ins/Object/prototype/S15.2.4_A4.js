// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since the Object prototype object is not a function, it has not
    [[create]] method
es5id: 15.2.4_A4
description: Checking if creating "new Object.prototype" fails
---*/

assert.throws(TypeError, function() {
  new Object.prototype;
}, '`new Object.prototype` throws a TypeError exception');
