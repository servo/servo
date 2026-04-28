// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.12_A4_T3
description: Type(y) and Type(z) are string primitives
---*/

//CHECK#1
if (("1" ? "" : "1") !== "") {
  throw new Test262Error('#1: ("1" ? "" : "1") === ""');
}

//CHECK#2
var y = new String("1");
if (("1" ? y : "") !== y) {
  throw new Test262Error('#2: (var y = new String("1"); ("1" ? y : "") === y');
}

//CHECK#3
var y = new String("y");
if ((y ? y : "1") !== y) {
  throw new Test262Error('#3: (var y = new String("y"); (y ? y : "1") === y');
}
