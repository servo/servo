// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: RegExp instance has no [[Call]] internal method
es5id: 15.10.7_A1_T1
description: Checking if call of RegExp instance fails
---*/

try {
    throw new Test262Error('#1.1: /[^a]*/() throw TypeError. Actual: ' + (/[^a]*/()));
} catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
