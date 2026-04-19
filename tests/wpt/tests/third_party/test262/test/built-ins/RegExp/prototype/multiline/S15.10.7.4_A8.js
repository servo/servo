// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype multiline property has the attribute DontEnum
es5id: 15.10.7.4_A8
description: >
    Checking if enumerating the multiline property of RegExp.prototype
    fails
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('multiline'), true, '__re.hasOwnProperty(\'multiline\') must return true');

assert.sameValue(
  __re.propertyIsEnumerable('multiline'),
  false,
  '__re.propertyIsEnumerable(\'multiline\') must return false'
);

var count = 0
for (var p in __re){
  if (p==="multiline") {
    count++
  }   
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
