// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T21
description: >
    RegExp is /[a-z]n/ and tested string is x, where x is
    function(){}()
---*/

var __re = /[a-z]n/;

assert.sameValue(
  __re.test(function(){}()),
  __re.exec(function(){}()) !== null,
  '__re.test(function(){}()) must return __re.exec(function(){}()) !== null'
);
