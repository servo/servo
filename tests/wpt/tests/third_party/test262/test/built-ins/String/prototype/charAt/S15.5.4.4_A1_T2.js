// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charAt(pos)
es5id: 15.5.4.4_A1_T2
description: pos is equation with false and true, and instance is Boolean object
---*/

var __instance = new Boolean;

__instance.charAt = String.prototype.charAt;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.charAt(false) + __instance.charAt(true) + __instance.charAt(true + 1) !== "fal") {
  throw new Test262Error('#1: __instance = new Boolean; __instance.charAt = String.prototype.charAt;  __instance = new Boolean; __instance.charAt = String.prototype.charAt; __instance.charAt(false)+__instance.charAt(true)+__instance.charAt(true+1) === "fal". Actual: ' + __instance.charAt(false) + __instance.charAt(true) + __instance.charAt(true + 1));
}
//
//////////////////////////////////////////////////////////////////////////////
