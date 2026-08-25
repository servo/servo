// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Error.prototype property has the attributes {DontDelete}
es5id: 15.11.3.1_A1_T1
description: Checking if deleting the Error.prototype property fails
includes: [propertyHelper.js]
---*/

var proto = Error.prototype;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
verifyNotConfigurable(Error, "prototype");
try {
  assert.sameValue(delete Error.prototype, false);
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
  assert(e instanceof TypeError);
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (Error.prototype !== proto) {
  throw new Test262Error('#2: var proto=Error.prototype; delete Error.prototype; Error.prototype===proto. Actual: ' + Error.prototype);
}
//
//////////////////////////////////////////////////////////////////////////////

// TODO: Convert to verifyProperty() format.
