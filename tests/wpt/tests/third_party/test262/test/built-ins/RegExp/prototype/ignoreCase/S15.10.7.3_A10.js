// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype ignoreCase property does not have a set accessor
es5id: 15.10.7.3_A10
description: Checking if varying the ignoreCase property fails
includes: [propertyHelper.js]
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('ignoreCase'), true, '__re.hasOwnProperty(\'ignoreCase\') must return true');

var __sample = /a|b|c/;
var __obj = __sample.ignoreCase;

verifyNotWritable(__sample, "ignoreCase", "ignoreCase", "shifted");

assert.sameValue(
  __sample.ignoreCase,
  __obj,
  'The value of __sample.ignoreCase is expected to equal the value of __obj'
);
