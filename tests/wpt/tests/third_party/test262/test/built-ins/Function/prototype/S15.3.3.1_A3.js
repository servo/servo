// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The Function.prototype property has the attribute DontDelete
es5id: 15.3.3.1_A3
description: Checking if deleting the Function.prototype property fails
includes: [propertyHelper.js]
---*/

verifyNotConfigurable(Function, "prototype");

try {
  assert.sameValue(delete Function.prototype, false);
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
  assert(e instanceof TypeError);
}

if (!(Function.hasOwnProperty('prototype'))) {
  throw new Test262Error('#2: the Function.prototype property has the attributes DontDelete.');
}

// TODO: Convert to verifyProperty() format.
