// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If F contains any character other than 'g', 'i', or 'm', or if it
    contains the same one more than once, then throw a SyntaxError exception
es5id: 15.10.4.1_A5_T1
description: Checking if using "ii" as F leads to throwing the correct exception
---*/

try {
    throw new Test262Error('#1.1: new RegExp(undefined,"ii") throw SyntaxError. Actual: ' + (new RegExp(undefined,"ii")));
} catch (e) {
  assert.sameValue(
    e instanceof SyntaxError,
    true,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
