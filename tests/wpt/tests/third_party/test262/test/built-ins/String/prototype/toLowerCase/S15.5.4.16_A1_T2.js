// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T2
description: Instance is Boolean object
---*/

var __instance = new Boolean;

__instance.toLowerCase = String.prototype.toLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.toLowerCase() !== "false") {
  throw new Test262Error('#1: __instance = new Boolean; __instance.toLowerCase = String.prototype.toLowerCase;  __instance.toLowerCase() === "false". Actual: ' + __instance.toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
