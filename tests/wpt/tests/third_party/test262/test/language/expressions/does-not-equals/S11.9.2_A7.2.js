// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is Object and Type(y) is Boolean,
    return ToPrimitive(x) != y
es5id: 11.9.2_A7.2
description: x is object, y is primitive boolean
---*/

//CHECK#1
if ((new Boolean(true) != true) !== false) {
  throw new Test262Error('#1: (new Boolean(true) != true) === false');
}

//CHECK#2
if ((new Number(1) != true) !== false) {
  throw new Test262Error('#2: (new Number(1) != true) === false');
}

//CHECK#3
if ((new String("1") != true) !== false) {
  throw new Test262Error('#3: (new String("1") != true) === false');
}
