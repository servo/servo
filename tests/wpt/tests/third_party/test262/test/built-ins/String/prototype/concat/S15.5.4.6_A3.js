// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.concat([,[...]]) can't change the instance to be applied
es5id: 15.5.4.6_A3
description: Checking if varying the instance that is applied fails
---*/

var __instance = new String("one");

__instance.concat("two");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance != "one") {
  throw new Test262Error('#1: __instance = new String("one"); __instance.concat("two");  __instance = new String("one"); __instance.concat("two"); __instance == "one". Actual: ' + __instance);
}
//
//////////////////////////////////////////////////////////////////////////////
