// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T8
description: Call toLowerCase() function of Infinity
---*/

Number.prototype.toLowerCase = String.prototype.toLowerCase;

if (Infinity.toLowerCase() !== "infinity") {
  throw new Test262Error('#1: Number.prototype.toLowerCase = String.prototype.toLowerCase; Infinity.toLowerCase()=== "infinity". Actual: ' + Infinity.toLowerCase());
}
