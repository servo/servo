// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T14
description: RegExp is /AL|se/ and tested string is new Boolean
---*/

var __string = new Boolean;
var __re = /AL|se/;

assert.sameValue(
  __re.test(__string),
  __re.exec(__string) !== null,
  '__re.test(new Boolean) must return __re.exec(__string) !== null'
);
