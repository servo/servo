// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pattern and flags are defined, then
    call the RegExp constructor (15.10.4.1), passing it the pattern and flags arguments and return the object constructed by that constructor
es5id: 15.10.3.1_A3_T1
description: R is "d+" and instance is RegExp(R,"i")
---*/

var __re = "d+";
var __instance = RegExp(__re, "i");

assert.sameValue(
  __instance.constructor,
  RegExp,
  'The value of __instance.constructor is expected to equal the value of RegExp'
);

assert.sameValue(__instance.source, __re, 'The value of __instance.source is expected to equal the value of __re');
