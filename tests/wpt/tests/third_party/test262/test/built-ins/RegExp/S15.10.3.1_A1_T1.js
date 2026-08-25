// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pattern is an object R whose [[Class]] property is "RegExp" and flags
    is undefined, then return R unchanged
es5id: 15.10.3.1_A1_T1
description: R is /x/i and instance is RegExp(R)
---*/

var __re = /x/i;
var __instance = RegExp(__re);
__re.indicator = 1;

assert.sameValue(__instance.indicator, 1, 'The value of __instance.indicator is expected to be 1');
