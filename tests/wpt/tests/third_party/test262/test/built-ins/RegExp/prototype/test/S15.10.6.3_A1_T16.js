// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T16
description: RegExp is /undefined/ and call test() without arguments
---*/

var __re = /undefined/;

assert.sameValue(__re.test(), __re.exec() !== null, '__re.test() must return __re.exec() !== null');
