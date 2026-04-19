// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-boolean.prototype.tostring
info: |
    The toString function is not generic, it cannot be transferred
    to other kinds of objects for use as a method and there is should be
    a TypeError exception if its this value is not a Boolean object
es5id: 15.6.4.2_A2_T2
description: transferring to the Number objects
---*/

assert.throws(TypeError, () => {
  var s1 = new Number();
  s1.toString = Boolean.prototype.toString;
  s1.toString();
});

assert.throws(TypeError, () => {
  var s2 = new Number();
  s2.myToString = Boolean.prototype.toString;
  s2.myToString();
});
