// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T10
description: RegExp is /1|12/ and tested string is 1.01
---*/

var __string = 1.01;
var __re = /1|12/;

assert.sameValue(
  __re.test(__string),
  __re.exec(__string) !== null,
  '__re.test(1.01) must return __re.exec(__string) !== null'
);
