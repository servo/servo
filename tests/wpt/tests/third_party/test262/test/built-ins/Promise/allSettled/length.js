// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Promise.allSettled `length` property
info: |
  ES Section 17:
  Every built-in Function object, including constructors, has a length
  property whose value is an integer. Unless otherwise specified, this value
  is equal to the largest number of named arguments shown in the subclause
  headings for the function description, including optional parameters.

  [...]

  Unless otherwise specified, the length property of a built-in Function
  object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Promise.allSettled]
---*/

verifyProperty(Promise.allSettled, 'length', {
  configurable: true,
  writable: false,
  enumerable: false,
  value: 1,
});
