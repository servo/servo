// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator.prototype.compare
description: >
  The length of the bound Collator compare function is 2.
info: |
  get Intl.Collator.prototype.compare

  ...
  4. If collator.[[BoundCompare]] is undefined, then
    a. Let F be a new built-in function object as defined in 10.3.4.
    b. Let bc be BoundFunctionCreate(F, collator, « »).
    c. Perform ! DefinePropertyOrThrow(bc, "length", PropertyDescriptor {[[Value]]: 2,
       [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: true}).
    ...

includes: [propertyHelper.js]
---*/

var compareFn = new Intl.Collator().compare;

verifyProperty(compareFn, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
