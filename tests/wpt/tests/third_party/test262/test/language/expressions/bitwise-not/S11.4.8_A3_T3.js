// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator ~x returns ~ToInt32(x)
es5id: 11.4.8_A3_T3
description: Type(x) is string primitive or String object
---*/

//CHECK#1
if (~"1" !== -2) {
  throw new Test262Error('#1: ~"1" === -2. Actual: ' + (~"1"));
}

//CHECK#2
if (~new String("0") !== -1) {
  throw new Test262Error('#2: ~new String("0") === -1. Actual: ' + (~new String("0")));
}

//CHECK#3
if (~"x" !== -1) {
  throw new Test262Error('#3: ~"x" === -1. Actual: ' + (~"x"));
}

//CHECK#4
if (~"" !== -1) {
  throw new Test262Error('#4: ~"" === -1. Actual: ' + (~""));
}

//CHECK#5
if (~new String("-2") !== 1) {
  throw new Test262Error('#5: ~new String("-2") === 1. Actual: ' + (~new String("-2")));
}
