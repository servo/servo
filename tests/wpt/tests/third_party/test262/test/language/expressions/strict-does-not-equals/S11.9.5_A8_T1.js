// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is different from Type(y), return true
es5id: 11.9.5_A8_T1
description: x or y is primitive boolean
---*/

//CHECK#1
if (!(true !== new Boolean(true))) {
  throw new Test262Error('#1: true !== new Number(true)');
}

//CHECK#2
if (!(true !== 1)) {
  throw new Test262Error('#2: true !== 1');
}

//CHECK#3
if (!(true !== new Number(true))) {
  throw new Test262Error('#3: true !== new Number(true)');
}

//CHECK#4
if (!(true !== "1")) {
  throw new Test262Error('#4: true !== "1"');
}

//CHECK#5
if (!(true !== new String(true))) {
  throw new Test262Error('#5: true !== new String(true)');
}

//CHECK#6
if (!(new Boolean(false) !== false)) {
  throw new Test262Error('#6: new Number(false) !== false');
}

//CHECK#7
if (!(0 !== false)) {
  throw new Test262Error('#7: 0 !== false');
}

//CHECK#8
if (!(new Number(false) !== false)) {
  throw new Test262Error('#8: new Number(false) !== false');
}

//CHECK#9
if (!("0" !== false)) {
  throw new Test262Error('#9: "0" !== false');
}

//CHECK#10
if (!(false !== new String(false))) {
  throw new Test262Error('#10: false !== new String(false)');
}

//CHECK#11
if (!(true !== {valueOf: function () {return true}})) {
  throw new Test262Error('#11: true !== {valueOf: function () {return true}}');
}
