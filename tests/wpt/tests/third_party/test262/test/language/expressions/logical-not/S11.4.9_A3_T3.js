// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator !x returns !ToBoolean(x)
es5id: 11.4.9_A3_T3
description: Type(x) is string primitive or String object
---*/

//CHECK#1
if (!"1" !== false) {
  throw new Test262Error('#1: !"1" === false');
}

//CHECK#2
if (!new String("0") !== false) {
  throw new Test262Error('#2: !new String("0") === false');
}

//CHECK#3
if (!"x" !== false) {
  throw new Test262Error('#3: !"x" === false');
}

//CHECK#4
if (!"" !== true) {
  throw new Test262Error('#4: !"" === true');
}

//CHECK#5
if (!new String("") !== false) {
  throw new Test262Error('#5: !new String("") === false');
}
