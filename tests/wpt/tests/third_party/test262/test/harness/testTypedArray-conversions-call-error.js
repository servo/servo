// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including testTypedArray.js will expose:

        testTypedArrayConversions()

includes: [testTypedArray.js]
features: [TypedArray]
---*/
var threw = false;

try {
  testTypedArrayConversions({}, () => {});
} catch(err) {
  threw = true;
  if (err.constructor !== TypeError) {
    throw new Error(
      'Expected a TypeError, but a "' + err.constructor.name +
      '" was thrown.'
    );
  }
}

if (threw === false) {
  throw new Error('Expected a TypeError, but no error was thrown.');
}


