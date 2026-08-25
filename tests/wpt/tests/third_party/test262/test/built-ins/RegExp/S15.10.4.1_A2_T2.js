// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    pattern is an object R whose [[Class]] property is "RegExp" and flags
    is not undefined. If ToString(pattern) is not a valid flags arguments,
    then throw a SyntaxError exception
es5id: 15.10.4.1_A2_T2
description: >
    Checking if execution of "new RegExp(pattern, {})", where the
    pattern is "/1?1/mig", fails
---*/

try {
  throw new Test262Error('#1.1: new RegExp(/1?1/mig, {}) throw SyntaxError. Actual: ' + (new RegExp(/1?1/mig, {})));
} catch (e) {
  assert.sameValue(
    e instanceof SyntaxError,
    true,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
