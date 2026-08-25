// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLocaleLowerCase()
es5id: 15.5.4.17_A1_T2
description: Instance is Boolean object
---*/

var __instance = new Boolean;

__instance.toLocaleLowerCase = String.prototype.toLocaleLowerCase;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.toLocaleLowerCase() !== "false") {
  throw new Test262Error('#1: __instance = new Boolean; __instance.toLocaleLowerCase = String.prototype.toLocaleLowerCase;  __instance.toLocaleLowerCase() === "false". Actual: ' + __instance.toLocaleLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
