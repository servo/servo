// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T7
description: Arguments are symbol and undefined, and instance is String
---*/

//since ToInteger("e") yelds 0
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String(void 0).slice("e", undefined) !== "undefined") {
  throw new Test262Error('#1: String(void 0).slice("e",undefined) === "undefined". Actual: ' + String(void 0).slice("e", undefined));
}
//
//////////////////////////////////////////////////////////////////////////////
