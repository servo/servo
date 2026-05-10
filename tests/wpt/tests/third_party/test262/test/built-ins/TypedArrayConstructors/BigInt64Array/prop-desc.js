// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-objects
description: BigInt64Array property descriptor
info: |
  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(this, "BigInt64Array", {
  writable: true,
  enumerable: false,
  configurable: true
});
