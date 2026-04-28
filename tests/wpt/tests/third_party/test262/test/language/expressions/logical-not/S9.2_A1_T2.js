// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from undefined value is false
es5id: 9.2_A1_T2
description: >
    Undefined, void and others are converted to Boolean by implicit
    transformation
---*/

// CHECK#1
if (!(undefined) !== true) {
  throw new Test262Error('#1: !(undefined) === true. Actual: ' + (!(undefined)));
}

// CHECK#2
if (!(void 0) !== true) {
  throw new Test262Error('#2: !(undefined) === true. Actual: ' + (!(undefined)));
}

// CHECK#3
if (!(eval("var x")) !== true) {
  throw new Test262Error('#3: !(eval("var x")) === true. Actual: ' + (!(eval("var x"))));
}
