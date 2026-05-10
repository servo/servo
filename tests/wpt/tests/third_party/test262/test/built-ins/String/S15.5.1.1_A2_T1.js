// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If value is not supplied, the empty string "" is returned
es5id: 15.5.1.1_A2_T1
description: Call String()
---*/

var __str = String();

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof __str !== "string") {
  throw new Test262Error('#1: __str = String(); typeof __str === "string". Actual: typeof __str ===' + typeof __str);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__str !== "") {
  throw new Test262Error('#2: __str = String(); __str === "". Actual: __str ===' + __str);
}
//
//////////////////////////////////////////////////////////////////////////////
