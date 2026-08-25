// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.or
description: Testing descriptor property of Atomics.or
includes: [propertyHelper.js]
features: [Atomics]
---*/
verifyProperty(Atomics, 'or', {
  enumerable: false,
  writable: true,
  configurable: true,
});
