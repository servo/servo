// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.match (regexp)
es5id: 15.5.4.10_A1_T1
description: Arguments is true, and instance is object
---*/

var __instance = new Object(true);

__instance.match = String.prototype.match;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.match(true)[0] !== "true") {
  throw new Test262Error('#1: __instance = new Object(true); __instance.match = String.prototype.match;  __instance.match(true)[0] === "true". Actual: ' + __instance.match(true)[0]);
}
//
//////////////////////////////////////////////////////////////////////////////
