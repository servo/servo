// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-asyncdisposablestack.prototype.disposed
description: >
  Property type and descriptor.
info: |
  get AsyncDisposableStack.prototype.disposed

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [explicit-resource-management]
---*/

var descriptor = Object.getOwnPropertyDescriptor(AsyncDisposableStack.prototype, 'disposed');

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

verifyProperty(AsyncDisposableStack.prototype, 'disposed', {
  enumerable: false,
  configurable: true,
});
