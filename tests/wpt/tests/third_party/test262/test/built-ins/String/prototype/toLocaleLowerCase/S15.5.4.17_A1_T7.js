// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleLowerCase()
es5id: 15.5.4.17_A1_T7
description: Call toLocaleLowerCase() function of NaN
---*/

Number.prototype.toLocaleLowerCase = String.prototype.toLocaleLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (NaN.toLocaleLowerCase() !== "nan") {
  throw new Test262Error('#1: Number.prototype.toLocaleLowerCase = String.prototype.toLocaleLowerCase; NaN.toLocaleLowerCase()=== "nan". Actual: ' + NaN.toLocaleLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
