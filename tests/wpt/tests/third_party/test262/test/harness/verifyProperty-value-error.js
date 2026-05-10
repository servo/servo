// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Including propertyHelper.js will expose:

        verifyProperty()
        ...

includes: [propertyHelper.js]
---*/

var threw = false;
var object = Object.defineProperty({}, "prop", {
  value: 1
});

try {
  verifyProperty(object, "prop", {
    value: 2
  });
} catch(err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Error(
      'Expected a Test262Error, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }

  if (err.message !== "obj['prop'] descriptor value should be 2; obj['prop'] value should be 2") {
    throw new Error('The error thrown did not define the specified message');
  }
}

if (threw === false) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}
