// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If one expression is undefined or null and another is not, return false
es5id: 11.9.1_A6.2_T2
description: y is null or undefined, x is not
---*/

//CHECK#1
if ((false == undefined) !== false) {
  throw new Test262Error('#1: (false == undefined) === false');
}

//CHECK#2
if ((Number.NaN == undefined) !== false) {
  throw new Test262Error('#2: (Number.NaN == undefined) === false');
}

//CHECK#3
if (("undefined" == undefined) !== false) {
  throw new Test262Error('#3: ("undefined" == undefined) === false');
}

//CHECK#4
if (({} == undefined) !== false) {
  throw new Test262Error('#4: ({} == undefined) === false');
}

//CHECK#5
if ((false == null) !== false) {
  throw new Test262Error('#5: (false == null) === false');
}

//CHECK#6
if ((0 == null) !== false) {
  throw new Test262Error('#6: (0 == null) === false');
}

//CHECK#7
if (("null" == null) !== false) {
  throw new Test262Error('#7: ("null" == null) === false');
}

//CHECK#8
if (({} == null) !== false) {
  throw new Test262Error('#8: ({} == null) === false');
}
