// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp)
es5id: 15.5.4.12_A1_T1
description: Argument is true, and instance is object
---*/

var __instance = new Object(true);

__instance.search = String.prototype.search;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.search(true) !== 0) {
  throw new Test262Error('#1: __instance = new Object(true); __instance.search = String.prototype.search;  __instance.search(true) === 0. Actual: ' + __instance.search(true));
}
//
//////////////////////////////////////////////////////////////////////////////
