// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parsefloat-string
description: "StrWhiteSpaceChar :: NBSB (U+00A0)"
---*/

//CHECK#1
if (parseFloat("\u00A01.1") !== parseFloat("1.1")) {
  throw new Test262Error('#1: parseFloat("\\u00A01.1") === parseFloat("1.1"). Actual: ' + (parseFloat("\u00A01.1")));
}

//CHECK#2
if (parseFloat("\u00A0\u00A0-1.1") !== parseFloat("-1.1")) {
  throw new Test262Error('#2: parseFloat("\\u00A0\\u00A0-1.1") === parseFloat("-1.1"). Actual: ' + (parseFloat("\u00A0\u00A0-1.1")));
}

//CHECK#3
assert.sameValue(parseFloat("\u00A0"), NaN);
