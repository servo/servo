// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pattern is an object R whose [[Class]] property is "RegExp" and flags is defined, then
    call the RegExp constructor (15.10.4.1), passing it the pattern and flags arguments and return the object constructed by that constructor
es5id: 15.10.3.1_A2_T2
description: >
    Checking if using dafined variable "x = 1" as flags leads to
    throwing the correct exception
---*/

var x = 1;

try {
    throw new Test262Error('#1.1: var x = 1; RegExp(/[a-b]?/, x) throw SyntaxError. Actual: ' + (RegExp(/[a-b]?/, x)));
} catch (e) {
  assert.sameValue(
    e instanceof SyntaxError,
    true,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
