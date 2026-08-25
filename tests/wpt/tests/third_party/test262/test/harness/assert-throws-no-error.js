// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions that do not throw errors do not satisfy the assertion.
---*/

var threw = false;

try {
  assert.throws(Error, function() {});
} catch(err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Error(
      'Expected a Test262Error, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }
}

if (threw === false) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}
