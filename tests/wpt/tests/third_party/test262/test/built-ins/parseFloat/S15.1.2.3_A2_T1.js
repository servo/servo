// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parsefloat-string
description: "StrWhiteSpaceChar :: TAB (U+0009)"
---*/

//CHECK#1
if (parseFloat("\u00091.1") !== parseFloat("1.1")) {
  throw new Test262Error('#1: parseFloat("\\u00091.1") === parseFloat("1.1"). Actual: ' + (parseFloat("\u00091.1")));
}

//CHECK#2
if (parseFloat("\u0009\u0009-1.1") !== parseFloat("-1.1")) {
  throw new Test262Error('#2: parseFloat("\\u0009\\u0009-1.1") === parseFloat("-1.1"). Actual: ' + (parseFloat("\u0009\u0009-1.1")));
}

//CHECK#3
if (parseFloat("	1.1") !== parseFloat("1.1")) {
  throw new Test262Error('#3: parseFloat("	1.1") === parseFloat("1.1"). Actual: ' + (parseFloat("	1.1")));
}

//CHECK#4
if (parseFloat("			1.1") !== parseFloat("1.1")) {
  throw new Test262Error('#4: parseFloat("			1.1") === parseFloat("1.1"). Actual: ' + (parseFloat("			1.1")));
}

//CHECK#5
if (parseFloat("			\u0009			\u0009-1.1") !== parseFloat("-1.1")) {
  throw new Test262Error('#5: parseFloat("			\\u0009			\\u0009-1.1") === parseFloat("-1.1"). Actual: ' + (parseFloat("			\u0009			\u0009-1.1")));
}

//CHECK#6
assert.sameValue(parseFloat("\u0009"), NaN, "'\u0009'");
