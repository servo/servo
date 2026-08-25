// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.substring (start, end)
es5id: 15.5.4.15_A1_T7
description: Arguments are symbol and undefined, and instance is String
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String(void 0).substring("e", undefined) !== "undefined") {
  throw new Test262Error('#1: String(void 0).substring("e",undefined) === "undefined". Actual: ' + String(void 0).substring("e", undefined));
}
//
//////////////////////////////////////////////////////////////////////////////
