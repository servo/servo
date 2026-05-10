// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function.prototype.call can't be used as [[Construct]] caller
es5id: 15.3.4.4_A7_T6
description: >
    Checking if creating "new (Function("function
    f(){this.p1=1;};return f").call())" fails
---*/

try {
  var obj = new(Function("function f(){this.p1=1;};return f").call());
} catch (e) {
  throw new Test262Error('#1: Function.prototype.call can\'t be used as [[Construct]] caller');
}

assert.sameValue(obj.p1, 1, 'The value of obj.p1 is expected to be 1');
