// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is different from Type(y), return true
es5id: 11.9.5_A8_T2
description: x or y is primitive number
---*/

//CHECK#1
if (!(1 !== new Number(1))) {
  throw new Test262Error('#1: 1 !== new Number(1)');
}

//CHECK#2
if (!(1 !== true)) {
  throw new Test262Error('#2: 1 !== true');
}

//CHECK#3
if (!(1 !== new Boolean(1))) {
  throw new Test262Error('#3: 1 !== new Boolean(1)');
}

//CHECK#4
if (!(1 !== "1")) {
  throw new Test262Error('#4: 1 !== "1"');
}

//CHECK#5
if (!(1 !== new String(1))) {
  throw new Test262Error('#5: 1 !== new String(1)');
}

//CHECK#6
if (!(new Number(0) !== 0)) {
  throw new Test262Error('#6: new Number(0) !== 0');
}

//CHECK#7
if (!(false !== 0)) {
  throw new Test262Error('#7: false !== 0');
}

//CHECK#8
if (!(new Boolean(0) !== 0)) {
  throw new Test262Error('#8: new Boolean(0) !== 0');
}

//CHECK#9
if (!("0" !== 0)) {
  throw new Test262Error('#9: "0" !== 0');
}

//CHECK#10
if (!(new String(0) !== 0)) {
  throw new Test262Error('#10: new String(0) !== 0');
}

//CHECK#11
if (!(1 !== {valueOf: function () {return 1}})) {
  throw new Test262Error('#11: 1 !== {valueOf: function () {return 1}}');
}
