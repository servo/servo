// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function.prototype.call can't be used as [[Construct]] caller
es5id: 15.3.4.4_A7_T4
description: Checking if creating "new (Function("this.p1=1").call)" fails
---*/

try {
  var obj = new(Function("this.p1=1").call);
  throw new Test262Error('#1: Function.prototype.call can\'t be used as [[Construct]] caller');
} catch (e) {
  assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
}
