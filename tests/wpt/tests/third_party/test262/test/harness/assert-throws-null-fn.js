// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Fails if second arg is not a function
---*/

var threw = false;

try {
  assert.throws(TypeError, null);
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

threw = false;

try {
  assert.throws(TypeError, {});
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

threw = false;

try {
  assert.throws(TypeError, "");
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
