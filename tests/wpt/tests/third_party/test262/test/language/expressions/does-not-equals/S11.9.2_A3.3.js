// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(y) is Number and Type(y) is Boolean,
    return the result of comparison x != ToNumber(y)
es5id: 11.9.2_A3.3
description: x is primitive number, y is primitive boolean
---*/

//CHECK#1
if ((0 != false) !== false) {
  throw new Test262Error('#1: (0 != false) === false');
}

//CHECK#2
if (("1" != true) !== false) {
  throw new Test262Error('#2: ("1" != true) === false');
}

//CHECK#3
if ((new Boolean(false) != false) !== false) {
  throw new Test262Error('#3: (new Boolean(false) != false) === false');
}

//CHECK#4
if (({valueOf: function () {return "0"}} != false) !== false) {
  throw new Test262Error('#4: ({valueOf: function () {return "0"}} != false) === false');
}
