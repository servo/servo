// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) as well as Type(y) is undefined or null, return true
es5id: 11.9.1_A6.1
description: Checking all combinations
---*/

//CHECK#1
if ((undefined == undefined) !== true) {
  throw new Test262Error('#1: (undefined == undefined) === true');
}

//CHECK#2
if ((void 0 == undefined) !== true) {
  throw new Test262Error('#2: (void 0 == undefined) === true');
}

//CHECK#3
if ((undefined == eval("var x")) !== true) {
  throw new Test262Error('#3: (undefined == eval("var x")) === true');
}

//CHECK#4
if ((undefined == null) !== true) {
  throw new Test262Error('#4: (undefined == null) === true');
}

//CHECK#5
if ((null == void 0) !== true) {
  throw new Test262Error('#5: (null == void 0) === true');
}

//CHECK#6
if ((null == null) !== true) {
  throw new Test262Error('#6: (null == null) === true');
}
