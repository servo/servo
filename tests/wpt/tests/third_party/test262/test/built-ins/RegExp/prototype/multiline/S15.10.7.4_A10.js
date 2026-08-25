// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype multiline property does not have a set accessor
es5id: 15.10.7.4_A10
description: Checking if varying the multiline property fails
includes: [propertyHelper.js]
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('multiline'), true, '__re.hasOwnProperty(\'multiline\') must return true');

var __sample = /\n/;
var __obj = __sample.multiline;

verifyNotWritable(__sample, "multiline", "multiline", "shifted");

assert.sameValue(__sample.multiline, __obj, 'The value of __sample.multiline is expected to equal the value of __obj');
