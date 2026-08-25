// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: Testing descriptor property of Atomics.notify
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics, 'notify', {
  enumerable: false,
  writable: true,
  configurable: true,
});
