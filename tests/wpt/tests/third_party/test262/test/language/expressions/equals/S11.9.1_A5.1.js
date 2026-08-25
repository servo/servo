// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are String-s.
    Return true, if x and y are exactly the same sequence of characters; otherwise, return false
es5id: 11.9.1_A5.1
description: x and y are primitive string
---*/

//CHECK#1
if (("" == "") !== true) {
  throw new Test262Error('#1: ("" == "") === true');
}

//CHECK#2
if ((" " == " ") !== true) {
  throw new Test262Error('#2: " (" == " ") === true');
}

//CHECK#3
if ((" " == "") !== false) {
  throw new Test262Error('#3: " (" == "") === false');
}

//CHECK#4
if (("string" == "string") !== true) {
  throw new Test262Error('#4: ("string" == "string") === true');
}

//CHECK#5
if ((" string" == "string ") !== false) {
  throw new Test262Error('#5: (" string" == "string ") === false');
}

//CHECK#6
if (("1.0" == "1") !== false) {
  throw new Test262Error('#6: ("1.0" == "1") === false');
}

//CHECK#7
if (("0xff" == "255") !== false) {
  throw new Test262Error('#7: ("0xff" == "255") === false');
}
