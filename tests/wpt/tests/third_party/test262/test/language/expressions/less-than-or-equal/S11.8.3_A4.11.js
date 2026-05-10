// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is a prefix of y, return true
es5id: 11.8.3_A4.11
description: x and y are string primitives
---*/

//CHECK#1
if (("x" <= "x") !== true) {
  throw new Test262Error('#1: ("x" <= "x") === true');
}

//CHECK#2
if (("" <= "x") !== true) {
  throw new Test262Error('#2: ("" <= "x") === true');
}

//CHECK#3
if (("ab" <= "abcd") !== true) {
  throw new Test262Error('#3: ("ab" <= abcd") === true');
}

//CHECK#4
if (("abcd" <= "abc\u0064") !== true) {
  throw new Test262Error('#4: ("abcd" <= abc\\u0064") === true');
}

//CHECK#5
if (("x" <= "x" + "y") !== true) {
  throw new Test262Error('#5: ("x" <= "x" + "y") === true');
}

//CHECK#6
var x = "x";
if ((x <= x + "y") !== true) {
  throw new Test262Error('#6: var x = "x"; (x <= x + "y") === true');
}

//CHECK#7
if (("a\u0000" <= "a\u0000a") !== true) {
  throw new Test262Error('#7: ("a\\u0000" <= "a\\u0000a") === true');
}

//CHECK#8
if (("x" <= " x") !== false) {
  throw new Test262Error('#8: ("x" <= " x") === false');
}
