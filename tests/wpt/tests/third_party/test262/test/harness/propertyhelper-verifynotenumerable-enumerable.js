// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Objects whose specified string property is enumerable do not satisfy the
    assertion.
includes: [propertyHelper.js]
---*/
var threw = false;
var obj = {};
Object.defineProperty(obj, 'a', {
  enumerable: true
});

try {
  verifyNotEnumerable(obj, 'a');
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
  throw new Test262Error(
    'Expected a Test262Error, but no error was thrown for string key.'
  );
}
