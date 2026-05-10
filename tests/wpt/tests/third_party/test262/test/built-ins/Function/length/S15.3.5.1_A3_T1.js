// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: the length property has the attributes { ReadOnly }
es5id: 15.3.5.1_A3_T1
description: >
    Checking if varying the length property of
    Function("arg1,arg2,arg3","arg4,arg5", null) fails
includes: [propertyHelper.js]
---*/

var f = new Function("arg1,arg2,arg3", "arg4,arg5", null);

assert(f.hasOwnProperty('length'));

var flength = f.length;

verifyNotWritable(f, "length", null, function() {});

assert.sameValue(f.length, flength);

try {
  f.length();
  throw new Test262Error('#3: the function.length property has the attributes ReadOnly');
} catch (e) {
  if (e instanceof Test262Error) {
    throw e;
  }
}

if (f.length !== 5) {
  throw new Test262Error('#4: the length property has the attributes { ReadOnly }');
}

// TODO: Convert to verifyProperty() format.
