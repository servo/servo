// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The RegExp.prototype property has the attribute DontDelete
es5id: 15.10.5.1_A3
description: Checking if deleting the RegExp.prototype property fails
includes: [propertyHelper.js]
---*/
assert.sameValue(RegExp.hasOwnProperty('prototype'), true);

verifyNotConfigurable(RegExp, "prototype");

try {
  assert.sameValue(delete RegExp.prototype, false);
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
  assert(e instanceof TypeError);
}

if (RegExp.hasOwnProperty('prototype') !== true) {
    throw new Test262Error('#2: delete RegExp.prototype; RegExp.hasOwnProperty(\'prototype\') === true');
}

// TODO: Convert to verifyProperty() format.
// TODO: Convert to assert.throws() format.
