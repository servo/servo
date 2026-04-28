// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.xor
description: >
  Atomics.xor.name is "xor".
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics.xor, 'name', {
  value: 'xor',
  enumerable: false,
  writable: false,
  configurable: true,
});
