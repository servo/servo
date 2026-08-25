// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.xor
description: Testing descriptor property of Atomics.xor
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics, 'xor', {
  enumerable: false,
  writable: true,
  configurable: true,
});
