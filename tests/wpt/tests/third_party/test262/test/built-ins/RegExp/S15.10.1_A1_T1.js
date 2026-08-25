// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: RegExp syntax errors must be caught when matcher(s) compiles
es5id: 15.10.1_A1_T1
description: Tested RegExp is "a**"
---*/

try {
    throw new Test262Error('#1.1: new RegExp("a**") throw SyntaxError. Actual: ' + (new RegExp("a**")));
} catch (e) {
  assert.sameValue(
    e instanceof SyntaxError,
    true,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}

// TODO: Convert to assert.throws() format.
