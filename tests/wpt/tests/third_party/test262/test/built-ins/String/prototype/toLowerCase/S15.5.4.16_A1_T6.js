// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T6
description: Call toLowerCase() function of Number.NEGATIVE_INFINITY
---*/

Number.prototype.toLowerCase = String.prototype.toLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ((Number.NEGATIVE_INFINITY).toLowerCase() !== "-infinity") {
  throw new Test262Error('#1: Number.prototype.toLowerCase = String.prototype.toLowerCase; (Number.NEGATIVE_INFINITY).toLowerCase() === "-infinity". Actual: ' + (Number.NEGATIVE_INFINITY).toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
