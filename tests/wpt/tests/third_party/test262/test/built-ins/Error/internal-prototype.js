// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-error-constructor
description: >
  The Error constructor has a [[Prototype]] internal slot whose value is %Function.prototype%.
---*/

assert.sameValue(
  Function.prototype.isPrototypeOf(Error().constructor),
  true,
  'Function.prototype.isPrototypeOf(err1.constructor) returns true'
);

assert.sameValue(
  Function.prototype.isPrototypeOf(Error.constructor),
  true,
  'Function.prototype.isPrototypeOf(Error.constructor) returns true'
);
