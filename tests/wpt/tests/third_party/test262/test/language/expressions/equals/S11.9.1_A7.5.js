// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is Number and Type(y) is Object,
    return x == ToPrimitive(y)
es5id: 11.9.1_A7.5
description: y is object, x is primitive number
---*/

//CHECK#1
if ((1 == new Boolean(true)) !== true) {
  throw new Test262Error('#1: (1 == new Boolean(true)) === true');
}

//CHECK#2
if ((-1 == new Number(-1)) !== true) {
  throw new Test262Error('#2: (-1 == new Number(-1)) === true');
}

//CHECK#3
if ((-1 == new String("-1")) !== true) {
  throw new Test262Error('#3: (-1 == new String("-1")) === true');
}
