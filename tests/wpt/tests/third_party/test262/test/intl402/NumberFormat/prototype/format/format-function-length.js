// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: >
  The length of the bound Number Format function is 1.
info: |
  get Intl.NumberFormat.prototype.format

  ...
  4. If nf.[[BoundFormat]] is undefined, then
    a. Let F be a new built-in function object as defined in Number Format Functions (11.1.4).
    b. Let bf be BoundFunctionCreate(F, nf, « »).
    c. Perform ! DefinePropertyOrThrow(bf, "length", PropertyDescriptor {[[Value]]: 1,
       [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true}).
    ...

includes: [propertyHelper.js]
---*/

var formatFn = new Intl.NumberFormat().format;

verifyProperty(formatFn, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
