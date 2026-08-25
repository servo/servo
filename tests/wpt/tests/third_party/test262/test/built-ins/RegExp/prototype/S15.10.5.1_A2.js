// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype property has the attribute DontEnum
es5id: 15.10.5.1_A2
description: Checking if enumerating the RegExp.prototype property fails
---*/
assert.sameValue(RegExp.hasOwnProperty('prototype'), true, 'RegExp.hasOwnProperty(\'prototype\') must return true');

assert.sameValue(
  RegExp.propertyIsEnumerable('prototype'),
  false,
  'RegExp.propertyIsEnumerable(\'prototype\') must return false'
);

var count=0;
for (var p in RegExp){
    if (p==="prototype") {
      count++;
    }
}

assert.sameValue(count, 0, 'The value of count is expected to be 0');

// TODO: Convert to verifyProperty() format.
