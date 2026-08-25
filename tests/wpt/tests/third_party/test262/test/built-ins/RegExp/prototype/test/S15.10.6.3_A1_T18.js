// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T18
description: RegExp is /nd|ne/ and tested string is undefined
---*/

var __re = /nd|ne/;

assert.sameValue(
  __re.test(undefined),
  __re.exec(undefined) !== null,
  '__re.test(undefined) must return __re.exec(undefined) !== null'
);
