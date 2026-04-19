// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.concat([,[...]])
es5id: 15.5.4.6_A1_T8
description: >
    Call concat([,[...]]) function with void 0 argument of string
    object
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(void 0) evaluates to "undefined" concat(void 0) evaluates to concat("undefined")
if (String(42).concat(void 0) !== "42undefined") {
  throw new Test262Error('#1: String(42).concat(void 0) === "42undefined". Actual: ' + String(42).concat(void 0));
}
//
//////////////////////////////////////////////////////////////////////////////
