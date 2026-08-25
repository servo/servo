// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype
description: >
  The initial value of Error.prototype is the Error prototype object.
includes: [propertyHelper.js]
---*/

assert.sameValue(
  Error.prototype.isPrototypeOf(new Error()), true,
  'Error.prototype.isPrototypeOf(new Error()) returns true'
);

assert.sameValue(
  Error.prototype.isPrototypeOf(Error()), true,
  'Error.prototype.isPrototypeOf(Error()) returns true'
);

verifyProperty(Error, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false,
});
