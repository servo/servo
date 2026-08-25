// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T17
description: RegExp is /ll|l/ and tested string is null
---*/

var __re = /ll|l/;

assert.sameValue(__re.test(null), __re.exec(null) !== null, '__re.test(null) must return __re.exec(null) !== null');
