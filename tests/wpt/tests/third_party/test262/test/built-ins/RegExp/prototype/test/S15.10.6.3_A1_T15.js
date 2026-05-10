// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T15
description: >
    RegExp is /LS/i and tested string is {toString:function(){return
    false;}}
---*/

var __string = {toString:function(){return false;}};
var __re = /LS/i;

assert.sameValue(
  __re.test(__string),
  __re.exec(__string) !== null,
  '__re.test({toString:function(){return false;}}) must return __re.exec(__string) !== null'
);
