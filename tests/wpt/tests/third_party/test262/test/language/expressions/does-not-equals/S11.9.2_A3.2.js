// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is Boolean and Type(y) is Number,
    return the result of comparison ToNumber(x) != y
es5id: 11.9.2_A3.2
description: x is primitive boolean, y is primitive number
---*/

//CHECK#1
if ((true != 1) !== false) {
  throw new Test262Error('#1: (true != 1) === false');
}

//CHECK#2
if ((false != "0") !== false) {
  throw new Test262Error('#2: (false != "0") === false');
}

//CHECK#3
if ((true != new Boolean(true)) !== false) {
  throw new Test262Error('#3: (true != new Boolean(true)) === false');
}

//CHECK#4
if ((true != {valueOf: function () {return 1}}) !== false) {
  throw new Test262Error('#4: (true != {valueOf: function () {return 1}}) === false');
}
