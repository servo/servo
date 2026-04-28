// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toUpperCase()
es5id: 15.5.4.18_A1_T7
description: Call toUpperCase() function of NaN
---*/

Number.prototype.toUpperCase = String.prototype.toUpperCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (NaN.toUpperCase() !== "NAN") {
  throw new Test262Error('#1: Number.prototype.toUpperCase = String.prototype.toUpperCase; NaN.toUpperCase()=== "NAN". Actual: ' + NaN.toUpperCase());
}
//
//////////////////////////////////////////////////////////////////////////////
