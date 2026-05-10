// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) as well as Type(y) is Undefined or Null, return true
es5id: 11.9.2_A6.1
description: Checking all combinations
---*/

//CHECK#1
if ((undefined != undefined) !== false) {
  throw new Test262Error('#1: (undefined != undefined) === false');
}

//CHECK#2
if ((void 0 != undefined) !== false) {
  throw new Test262Error('#2: (void 0 != undefined) === false');
}

//CHECK#3
if ((undefined != eval("var x")) !== false) {
  throw new Test262Error('#3: (undefined != eval("var x")) === false');
}

//CHECK#4
if ((undefined != null) !== false) {
  throw new Test262Error('#4: (undefined != null) === false');
}

//CHECK#5
if ((null != void 0) !== false) {
  throw new Test262Error('#5: (null != void 0) === false');
}

//CHECK#6
if ((null != null) !== false) {
  throw new Test262Error('#6: (null != null) === false');
}
