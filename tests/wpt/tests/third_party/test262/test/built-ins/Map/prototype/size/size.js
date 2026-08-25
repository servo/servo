// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-map.prototype.size
description: >
  Property type and descriptor.
info: |
  get Map.prototype.size

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Map.prototype, 'size');

assert.sameValue(
  typeof descriptor.get,
  'function',
  'typeof descriptor.get is function'
);
assert.sameValue(
  typeof descriptor.set,
  'undefined',
  'typeof descriptor.set is undefined'
);

verifyNotEnumerable(Map.prototype, 'size');
verifyConfigurable(Map.prototype, 'size');
