// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype ignoreCase property has the attribute DontEnum
es5id: 15.10.7.3_A8
description: >
    Checking if enumerating the ignoreCase property of
    RegExp.prototype fails
---*/

var __re = RegExp.prototype;

assert.sameValue(__re.hasOwnProperty('ignoreCase'), true, '__re.hasOwnProperty(\'ignoreCase\') must return true');

assert.sameValue(
  __re.propertyIsEnumerable('ignoreCase'),
  false,
  '__re.propertyIsEnumerable(\'ignoreCase\') must return false'
);

var count = 0
for (var p in __re){
  if (p==="ignoreCase") {
    count++
  }   
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
