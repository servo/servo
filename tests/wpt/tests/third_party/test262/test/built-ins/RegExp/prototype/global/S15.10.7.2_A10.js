// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype global property does not have a set accessor
es5id: 15.10.7.2_A10
description: Checking if varying the global property fails
includes: [propertyHelper.js]
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('global'), true, '__re.hasOwnProperty(\'global\') must return true');

var __sample = /^|^/;
var __obj = __sample.global;

verifyNotWritable(__sample, "global", "global", "shifted");

assert.sameValue(__sample.global, __obj, 'The value of __sample.global is expected to equal the value of __obj');
