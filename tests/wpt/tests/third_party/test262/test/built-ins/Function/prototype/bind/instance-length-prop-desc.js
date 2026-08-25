// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.bind
description: >
  "length" property of a bound function has correct descriptor.
info: |
  Function.prototype.bind ( thisArg, ...args )

  [...]
  8. Perform ! SetFunctionLength(F, L).
  [...]

  SetFunctionLength ( F, length )

  [...]
  4. Return ! DefinePropertyOrThrow(F, "length", PropertyDescriptor { [[Value]]:
  length, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true }).
includes: [propertyHelper.js]
---*/

function fn() {}

verifyProperty(fn.bind(null), "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});
