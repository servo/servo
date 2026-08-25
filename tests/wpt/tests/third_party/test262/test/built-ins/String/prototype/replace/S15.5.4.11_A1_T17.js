// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T17
description: Instance is String object, searchValue is regular expression
---*/

var __re = new RegExp(x, "g");

var __instance = String("asdf");

var __str = "1";
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__instance.replace(__re, __str) !== "1a1s1d1f1") {
  throw new Test262Error('#1: var x; var __re = new RegExp(x,"g"); __instance = String("asdf"); __str = "1"; __instance.replace(__re, __str) === "1a1s1d1f1". Actual: ' + __instance.replace(__re, __str));
}
//
//////////////////////////////////////////////////////////////////////////////

var x;
