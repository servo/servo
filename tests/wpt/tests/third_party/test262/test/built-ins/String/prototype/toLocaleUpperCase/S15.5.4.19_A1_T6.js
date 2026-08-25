// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleUpperCase()
es5id: 15.5.4.19_A1_T6
description: Call toLocaleUpperCase() function of Number.NEGATIVE_INFINITY
---*/

Number.prototype.toLocaleUpperCase = String.prototype.toLocaleUpperCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ((Number.NEGATIVE_INFINITY).toLocaleUpperCase() !== "-INFINITY") {
  throw new Test262Error('#1: Number.prototype.toLocaleUpperCase = String.prototype.toLocaleUpperCase; (Number.NEGATIVE_INFINITY).toLocaleUpperCase() === "-INFINITY". Actual: ' + (Number.NEGATIVE_INFINITY).toLocaleUpperCase());
}
//
//////////////////////////////////////////////////////////////////////////////
