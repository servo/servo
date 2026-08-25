// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Objects whose specified symbol property is not enumerable do not satisfy the
    assertion.
includes: [propertyHelper.js]
features: [Symbol]
---*/
var threw = false;
var obj = {};
var s = Symbol('1');
Object.defineProperty(obj, s, {
  enumerable: false
});

try {
  verifyEnumerable(obj, s);
} catch(err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Test262Error(
      'Expected a Test262Error, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }
}

if (threw === false) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}
