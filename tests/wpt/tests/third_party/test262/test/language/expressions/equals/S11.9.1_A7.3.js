// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is Boolean and Type(y) is Object,
    return x == ToPrimitive(y)
es5id: 11.9.1_A7.3
description: y is object, x is primitive boolean
---*/

//CHECK#1
if ((true == new Boolean(true)) !== true) {
  throw new Test262Error('#1: (true == new Boolean(true)) === true');
}

//CHECK#2
if ((true == new Number(1)) !== true) {
  throw new Test262Error('#2: (true == new Number(1)) === true');
}

//CHECK#3
if ((true == new String("+1")) !== true) {
  throw new Test262Error('#3: (true == new String("+1")) === true');
}
