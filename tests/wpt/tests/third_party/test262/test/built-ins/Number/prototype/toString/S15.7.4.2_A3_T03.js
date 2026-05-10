// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "toString: radix should be an integer between 2 and 36"
es5id: 15.7.4.2_A3_T03
description: radix is null value
---*/

try {
  var n = Number.prototype.toString(null);
  throw new Test262Error('#1: Number.prototype.toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number()).toString(null);
  throw new Test262Error('#2: (new Number()).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(0)).toString(null);
  throw new Test262Error('#3: (new Number(0)).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(-1)).toString(null);
  throw new Test262Error('#4: (new Number(-1)).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(1)).toString(null);
  throw new Test262Error('#5: (new Number(1)).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.NaN)).toString(null);
  throw new Test262Error('#6: (new Number(Number.NaN)).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.POSITIVE_INFINITY)).toString(null);
  throw new Test262Error('#7: (new Number(Number.POSITIVE_INFINITY)).toString(null) should throw an Error');
}
catch (e) {}

try {
  var n = (new Number(Number.NEGATIVE_INFINITY)).toString(null);
  throw new Test262Error('#8: (new Number(Number.NEGATIVE_INFINITY)).toString(null) should throw an Error');
}
catch (e) {}
