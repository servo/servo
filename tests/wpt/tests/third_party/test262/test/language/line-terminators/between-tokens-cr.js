// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: CARRIAGE RETURN (U+000D) may occur between any two tokens
esid: sec-line-terminators
es5id: 7.3_A1.2_T2
description: Insert real CARRIAGE RETURN between tokens of var x=1
---*/

varx=1;

if (x !== 1) {
  throw new Test262Error('#1: var\\nx\\n=\\n1\\n; x === 1. Actual: ' + (x));
}
