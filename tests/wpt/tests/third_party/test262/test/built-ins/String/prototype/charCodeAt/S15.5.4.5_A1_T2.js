// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.charCodeAt(pos)
es5id: 15.5.4.5_A1_T2
description: pos is equation with false and true, and instance is Boolean object
---*/

var __instance = new Boolean;

__instance.charCodeAt = String.prototype.charCodeAt;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.charCodeAt(false) !== 0x66) {
  throw new Test262Error('#1: __instance = new Boolean; __instance.charCodeAt = String.prototype.charCodeAt; __instance.charCodeAt(false)===0x66. Actual: ' + __instance.charCodeAt(false));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__instance.charCodeAt(true) !== 0x61) {
  throw new Test262Error('#2: __instance = new Boolean; __instance.charCodeAt = String.prototype.charCodeAt; __instance.charCodeAt(true)===0x61. Actual: ' + __instance.charCodeAt(true));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if (__instance.charCodeAt(true + 1) !== 0x6C) {
  throw new Test262Error('#3: __instance = new Boolean; __instance.charCodeAt = String.prototype.charCodeAt; __instance.charCodeAt(true+1) === 0x6C. Actual: ' + __instance.charCodeAt(true + 1));
}
//
//////////////////////////////////////////////////////////////////////////////
