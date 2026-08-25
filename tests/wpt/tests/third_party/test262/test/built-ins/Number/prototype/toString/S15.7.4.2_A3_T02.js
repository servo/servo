// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "toString: radix should be an integer between 2 and 36"
es5id: 15.7.4.2_A3_T02
description: radix is 37
---*/

try {
  var n = Number.prototype.toString(37);
  throw new Test262Error('#1: Number.prototype.toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number()).toString(37);
  throw new Test262Error('#2: (new Number()).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(0)).toString(37);
  throw new Test262Error('#3: (new Number(0)).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(-1)).toString(37);
  throw new Test262Error('#4: (new Number(-1)).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(1)).toString(37);
  throw new Test262Error('#5: (new Number(1)).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.NaN)).toString(37);
  throw new Test262Error('#6: (new Number(Number.NaN)).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.POSITIVE_INFINITY)).toString(37);
  throw new Test262Error('#7: (new Number(Number.POSITIVE_INFINITY)).toString(37) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.NEGATIVE_INFINITY)).toString(37);
  throw new Test262Error('#8: (new Number(Number.NEGATIVE_INFINITY)).toString(37) should throw an Error');
}
catch (e) {}
