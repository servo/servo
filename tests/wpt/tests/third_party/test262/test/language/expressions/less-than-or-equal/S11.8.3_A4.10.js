// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If y is a prefix of x and x !== y, return false
es5id: 11.8.3_A4.10
description: x and y are string primitives
---*/

//CHECK#1
if (("x " <= "x") !== false) {
  throw new Test262Error('#1: ("x " <= "x") === false');
}

//CHECK#2
if (("x" <= "") !== false) {
  throw new Test262Error('#2: ("x" <= "") === false');
}

//CHECK#3
if (("abcd" <= "ab") !== false) {
  throw new Test262Error('#3: ("abcd" <= ab") === false');
}

//CHECK#4
if (("abc\u0064" <= "abcd") !== true) {
  throw new Test262Error('#4: ("abc\\u0064" <= abcd") === true');
}

//CHECK#5
if (("x" + "y" <= "x") !== false) {
  throw new Test262Error('#5: ("x" + "y" <= "x") === false');
}

//CHECK#6
var x = "x";
if ((x + 'y' <= x) !== false) {
  throw new Test262Error('#6: var x = "x"; (x + "y" <= x) === false');
}
