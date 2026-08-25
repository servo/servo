// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If one expression is undefined or null and another is not, return false
es5id: 11.9.1_A6.2_T1
description: x is null or undefined, y is not
---*/

//CHECK#1
if ((undefined == true) !== false) {
  throw new Test262Error('#1: (undefined == true) === false');
}

//CHECK#2
if ((undefined == 0) !== false) {
  throw new Test262Error('#2: (undefined == 0) === false');
}

//CHECK#3
if ((undefined == "undefined") !== false) {
  throw new Test262Error('#3: (undefined == "undefined") === false');
}

//CHECK#4
if ((undefined == {}) !== false) {
  throw new Test262Error('#4: (undefined == {}) === false');
}

//CHECK#5
if ((null == false) !== false) {
  throw new Test262Error('#5: (null == false) === false');
}

//CHECK#6
if ((null == 0) !== false) {
  throw new Test262Error('#6: (null == 0) === false');
}

//CHECK#7
if ((null == "null") !== false) {
  throw new Test262Error('#7: (null == "null") === false');
}

//CHECK#8
if ((null == {}) !== false) {
  throw new Test262Error('#8: (null == {}) === false');
}
