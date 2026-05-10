// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pattern and flags are defined, then
    call the RegExp constructor (15.10.4.1), passing it the pattern and flags arguments and return the object constructed by that constructor
es5id: 15.10.3.1_A3_T2
description: >
    R is {toString:function(){return "[a-c]*";}} and instance is
    RegExp(R,"gm")
---*/

var __instance = RegExp({toString:function(){return "[a-c]*";}}, "gm");

assert.sameValue(
  __instance.constructor,
  RegExp,
  'The value of __instance.constructor is expected to equal the value of RegExp'
);

assert.sameValue(__instance.source, "[a-c]*", 'The value of __instance.source is expected to be "[a-c]*"');
