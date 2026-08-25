// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is different from Type(y), return true
es5id: 11.9.5_A8_T3
description: x or y is primitive string
---*/

//CHECK#1
if (!("1" !== new String("1"))) {
  throw new Test262Error('#1: "1" !== new String("1")');
}

//CHECK#2
if (!("1" !== true)) {
  throw new Test262Error('#2: "1" !== true');
}

//CHECK#3
if (!("1" !== new Boolean("1"))) {
  throw new Test262Error('#3: "1" !== new Boolean("1")');
}

//CHECK#4
if (!("1" !== 1)) {
  throw new Test262Error('#4: "1" === 1');
}

//CHECK#5
if (!("1" !== new Number("1"))) {
  throw new Test262Error('#5: "1" === new Number("1")');
}

//CHECK#6
if (!(new String(false) !== false)) {
  throw new Test262Error('#6: new Number(false) !== false');
}

//CHECK#7
if (!(false !== "0")) {
  throw new Test262Error('#7: false !== "0"');
}

//CHECK#8
if (!("0" !== new Boolean("0"))) {
  throw new Test262Error('#8: "0" !== new Boolean("0")');
}

//CHECK#9
if (!(false !== 0)) {
  throw new Test262Error('#9: false !== 0');
}

//CHECK#10
if (!(false !== new Number(false))) {
  throw new Test262Error('#10: false !== new Number(false)');
}

//CHECK#11
if (!("1" !== {valueOf: function () {return "1"}})) {
  throw new Test262Error('#11: "1" !== {valueOf: function () {return "1"}}');
}
