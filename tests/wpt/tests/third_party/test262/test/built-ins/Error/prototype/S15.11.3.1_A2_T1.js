// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Error.prototype property has the attributes {DontEnum}
es5id: 15.11.3.1_A2_T1
description: Checking if enumerating the Error.prototype property fails
---*/
assert(Error.hasOwnProperty('prototype'), 'Error.hasOwnProperty(\'prototype\') must return true');

assert(
  !Error.propertyIsEnumerable('prototype'),
  'The value of !Error.propertyIsEnumerable(\'prototype\') is expected to be true'
);

//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
// CHECK#2
var cout = 0;

for (var p in Error) {
  if (p === "prototype") {
    cout++;
  }
}

assert.sameValue(cout, 0, 'The value of cout is expected to be 0');

// TODO: Convert to verifyProperty() format.
