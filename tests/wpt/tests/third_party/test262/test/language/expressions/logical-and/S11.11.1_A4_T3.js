// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.11.1_A4_T3
description: Type(x) and Type(y) vary between primitive string and String object
---*/

//CHECK#1
if (("0" && "-1") !== "-1") {
  throw new Test262Error('#-1: ("0" && "-1") === "-1"');
}

//CHECK#2
if (("-1" && "x") !== "x") {
  throw new Test262Error('#2: ("-1" && "x") === "x"');
}

//CHECK#3
var y = new String(-1);
if ((new String("-1") && y) !== y) {
  throw new Test262Error('#3: (var y = new String(-1); (new String("-1") && y) === y');
}

//CHECK#4
var y = new String(NaN);
if ((new String("0") && y) !== y) {
  throw new Test262Error('#4: (var y = new String(NaN); (new String("0") && y) === y');
}

//CHECK#5
var y = new String("-x");
if ((new String("x") && y) !== y) {
  throw new Test262Error('#5: (var y = new String("-x"); (new String("x") && y) === y');
}

//CHECK#6
var y = new String(-1);
if ((new String(NaN) && y) !== y) {
  throw new Test262Error('#6: (var y = new String(-1); (new String(NaN) && y) === y');
}
