// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-asyncdisposablestack-prototype-object
description: AsyncDisposableStack.prototype.constructor
info: |
  The initial value of AsyncDisposableStack.prototype.constructor is the intrinsic object %AsyncDisposableStack%.

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [explicit-resource-management]
---*/

verifyProperty(AsyncDisposableStack.prototype, 'constructor', {
  value: AsyncDisposableStack,
  writable: true,
  enumerable: false,
  configurable: true
});
