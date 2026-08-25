// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.format
description: >
  The length of the bound DateTime Format function is 1.
info: |
  get Intl.DateTimeFormat.prototype.format

  ...
  4. If dtf.[[BoundFormat]] is undefined, then
    a. Let F be a new built-in function object as defined in DateTime Format Functions (12.1.5).
    b. Let bf be BoundFunctionCreate(F, dft, « »).
    c. Perform ! DefinePropertyOrThrow(bf, "length", PropertyDescriptor {[[Value]]: 1,
       [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true}).
    ...

includes: [propertyHelper.js]
---*/

var formatFn = new Intl.DateTimeFormat().format;

verifyProperty(formatFn, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
