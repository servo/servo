// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Array.prototype.findLast.name value and descriptor.
info: |
  Array.prototype.findLast ( predicate [ , thisArg ] )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [array-find-from-last]
---*/

assert.sameValue(
  Array.prototype.findLast.name, 'findLast',
  'The value of `Array.prototype.findLast.name` is `"findLast"`'
);

verifyProperty(Array.prototype.findLast, "name", {
  enumerable: false,
  writable: false,
  configurable: true
});
