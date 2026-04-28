// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-asyncdisposablestack.prototype.disposed
description: >
  AsyncDisposableStack.prototype.disposed.name value and descriptor.
info: |
  get AsyncDisposableStack.prototype.size

  17 ECMAScript Standard Built-in Objects

  Functions that are specified as get or set accessor functions of built-in
  properties have "get " or "set " prepended to the property name string.

includes: [propertyHelper.js]
features: [explicit-resource-management]
---*/

var descriptor = Object.getOwnPropertyDescriptor(AsyncDisposableStack.prototype, 'disposed');

assert.sameValue(descriptor.get.name,
  'get disposed',
  'The value of `descriptor.get.name` is `get disposed`'
);

verifyProperty(descriptor.get, 'name', {
  writable: false,
  enumerable: false,
  configurable: true,
});
