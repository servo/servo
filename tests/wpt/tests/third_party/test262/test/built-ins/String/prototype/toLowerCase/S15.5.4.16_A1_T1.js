// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T1
description: Arguments is true, and instance is object
---*/

var __instance = new Object(true);

__instance.toLowerCase = String.prototype.toLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.toLowerCase() !== "true") {
  throw new Test262Error('#1: __instance = new Object(true); __instance.toLowerCase = String.prototype.toLowerCase;  __instance.toLowerCase() === "true". Actual: ' + __instance.toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
