// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: RegExp instance type is RegExp
es5id: 15.10.7_A3_T1
description: >
    Checking type of RegExp instance with operators typeof, instanceof
    and check it constructor.  RegExp instance is /[^a]* /
---*/

var __re = /[^a]*/;

assert.sameValue(typeof __re, "object", 'The value of `typeof __re` is expected to be "object"');
assert.sameValue(__re.constructor, RegExp, 'The value of __re.constructor is expected to equal the value of RegExp');

assert.sameValue(
  __re instanceof RegExp,
  true,
  'The result of evaluating (__re instanceof RegExp) is expected to be true'
);
