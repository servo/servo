// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.add
description: Testing descriptor property of Atomics.add
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics, 'add', {
  enumerable: false,
  writable: true,
  configurable: true,
});
