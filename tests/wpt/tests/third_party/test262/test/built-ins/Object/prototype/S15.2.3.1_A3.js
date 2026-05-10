// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.1_A3
description: Checking if deleting "Object.prototype" property fails;
includes: [propertyHelper.js]
---*/

verifyNotConfigurable(Object, "prototype");

try {
  assert.sameValue(delete Object.prototype, false, 'The value of `delete Object.prototype` is expected to be false');
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
  assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
}

if (!(Object.hasOwnProperty('prototype'))) {
  throw new Test262Error('#2: the Object.prototype property has the attributes DontDelete.');
}

// TODO: Convert to verifyProperty() format.
