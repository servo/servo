// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including promiseHelper.js will expose a function:

        checkSequence

    To ensure execution order of some async chain, checkSequence accepts an array
    of numbers, each added during some operation, and verifies that they
    are in numeric order.

includes: [promiseHelper.js]
---*/

assert(checkSequence([1, 2, 3, 4, 5]));

var threw = false;

try {
  checkSequence([2, 1, 3, 4, 5]);
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

