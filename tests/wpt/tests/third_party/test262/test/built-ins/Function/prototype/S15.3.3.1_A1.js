// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Function.prototype property has the attribute ReadOnly
es5id: 15.3.3.1_A1
description: Checking if varying the Function.prototype property fails
includes: [propertyHelper.js]
---*/

var obj = Function.prototype;

verifyNotWritable(Function, "prototype", null, function() {
  return "shifted";
});

assert.sameValue(Function.prototype, obj, 'The value of Function.prototype is expected to equal the value of obj');

try {
  assert.sameValue(Function.prototype(), undefined, 'Function.prototype() returns undefined');
} catch (e) {
  throw new Test262Error('#2.1: the Function.prototype property has the attributes ReadOnly: ' + e);
}

// TODO: Convert to verifyProperty() format.
