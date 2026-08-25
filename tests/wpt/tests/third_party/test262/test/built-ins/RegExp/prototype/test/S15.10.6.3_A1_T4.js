// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T4
description: >
    RegExp is /a[a-z]{2,4}?/ and tested string is
    {toString:function(){return "abcdefghi";}}
---*/

var __string = {toString:function(){return "abcdefghi";}};
var __re = /a[a-z]{2,4}?/;

assert.sameValue(
  __re.test(__string),
  __re.exec(__string) !== null,
  '__re.test({toString:function(){return "abcdefghi";}}) must return __re.exec(__string) !== null'
);
