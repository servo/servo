// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If P's characters do not have the form Pattern, then throw a SyntaxError
    exception
es5id: 15.10.4.1_A9_T3
description: Pattern is "[a--z]"
---*/

try {
    throw new Test262Error('#1.1: new RegExp("[a--z]") throw SyntaxError. Actual: ' + (new RegExp("[a--z]")));
} catch (e) {
  assert.sameValue(
    e instanceof SyntaxError,
    true,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
