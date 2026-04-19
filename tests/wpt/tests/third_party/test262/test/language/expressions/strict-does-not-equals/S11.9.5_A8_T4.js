// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is different from Type(y), return true
es5id: 11.9.5_A8_T4
description: x or y is null or undefined
---*/

//CHECK#1
if (!(undefined !== null)) {
  throw new Test262Error('#1: undefined !== null');
}

//CHECK#2
if (!(null !== undefined)) {
  throw new Test262Error('#2: null !== undefined');
}

//CHECK#3
if (!(null !== 0)) {
  throw new Test262Error('#3: null !== 0');
}

//CHECK#4
if (!(0 !== null)) {
  throw new Test262Error('#4: 0 !== null');
}

//CHECK#5
if (!(null !== false)) {
  throw new Test262Error('#5: null !== false');
}

//CHECK#6
if (!(false !== null)) {
  throw new Test262Error('#6: false !== null');
}

//CHECK#7
if (!(undefined !== false)) {
  throw new Test262Error('#7: undefined !== false');
}

//CHECK#8
if (!(false !== undefined)) {
  throw new Test262Error('#8: false !== undefined');
}

//CHECK#9
if (!(null !== new Object())) {
  throw new Test262Error('#9: null !== new Object()');
}

//CHECK#10
if (!(new Object() !== null)) {
  throw new Test262Error('#10: new Object() !== null');
}

//CHECK#11
if (!(null !== "null")) {
  throw new Test262Error('#11: null !== "null"');
}

//CHECK#12
if (!("null" !== null)) {
  throw new Test262Error('#12: "null" !== null');
}

//CHECK#13
if (!(undefined !== "undefined")) {
  throw new Test262Error('#13: undefined !== "undefined"');
}

//CHECK#14
if (!("undefined" !== undefined)) {
  throw new Test262Error('#14: "undefined" !== undefined');
}
