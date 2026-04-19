// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.load
description: Testing descriptor property of Atomics.load
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics, 'load', {
  enumerable: false,
  writable: true,
  configurable: true,
});
