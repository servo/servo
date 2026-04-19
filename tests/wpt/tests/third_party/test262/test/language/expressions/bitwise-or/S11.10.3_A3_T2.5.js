// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x | y returns ToNumber(x) | ToNumber(y)
es5id: 11.10.3_A3_T2.5
description: >
    Type(x) is different from Type(y) and both types vary between
    String (primitive or object) and Boolean (primitive and object)
---*/

//CHECK#1
if ((true | "1") !== 1) {
  throw new Test262Error('#1: (true | "1") === 1. Actual: ' + ((true | "1")));
}

//CHECK#2
if (("1" | true) !== 1) {
  throw new Test262Error('#2: ("1" | true) === 1. Actual: ' + (("1" | true)));
}

//CHECK#3
if ((new Boolean(true) | "1") !== 1) {
  throw new Test262Error('#3: (new Boolean(true) | "1") === 1. Actual: ' + ((new Boolean(true) | "1")));
}

//CHECK#4
if (("1" | new Boolean(true)) !== 1) {
  throw new Test262Error('#4: ("1" | new Boolean(true)) === 1. Actual: ' + (("1" | new Boolean(true))));
}

//CHECK#5
if ((true | new String("1")) !== 1) {
  throw new Test262Error('#5: (true | new String("1")) === 1. Actual: ' + ((true | new String("1"))));
}

//CHECK#6
if ((new String("1") | true) !== 1) {
  throw new Test262Error('#6: (new String("1") | true) === 1. Actual: ' + ((new String("1") | true)));
}

//CHECK#7
if ((new Boolean(true) | new String("1")) !== 1) {
  throw new Test262Error('#7: (new Boolean(true) | new String("1")) === 1. Actual: ' + ((new Boolean(true) | new String("1"))));
}

//CHECK#8
if ((new String("1") | new Boolean(true)) !== 1) {
  throw new Test262Error('#8: (new String("1") | new Boolean(true)) === 1. Actual: ' + ((new String("1") | new Boolean(true))));
}
