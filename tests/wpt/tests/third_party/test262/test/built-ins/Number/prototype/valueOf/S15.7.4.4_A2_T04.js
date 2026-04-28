// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The valueOf function is not generic, it cannot be transferred
    to other kinds of objects for use as a method and there is should be
    a TypeError exception if its this value is not a Number object
es5id: 15.7.4.4_A2_T04
description: transferring to the Object objects
---*/

try {
  var s1 = new Object();
  s1.valueOf = Number.prototype.valueOf;
  var v1 = s1.valueOf();
  throw new Test262Error('#1: Number.prototype.valueOf on not a Number object should throw TypeError');
}
catch (e) {
  assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
}

try {
  var s2 = new Object();
  s2.myValueOf = Number.prototype.valueOf;
  var v2 = s2.myValueOf();
  throw new Test262Error('#2: Number.prototype.valueOf on not a Number object should throw TypeError');
}
catch (e) {
  assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
}
