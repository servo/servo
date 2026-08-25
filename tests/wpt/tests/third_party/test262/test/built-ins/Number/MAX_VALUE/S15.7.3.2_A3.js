// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.MAX_VALUE is DontDelete
es5id: 15.7.3.2_A3
description: Checking if deleting Number.MAX_VALUE fails
includes: [propertyHelper.js]
---*/

verifyNotConfigurable(Number, "MAX_VALUE");

// CHECK#1
try {
  assert.sameValue(delete Number.MAX_VALUE, false);
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
  assert(e instanceof TypeError);
}

// TODO: Convert to verifyProperty() format.
