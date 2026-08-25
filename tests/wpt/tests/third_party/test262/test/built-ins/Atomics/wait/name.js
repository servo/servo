// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  Atomics.wait.name is "wait".
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics.wait, 'name', {
  value: 'wait',
  enumerable: false,
  writable: false,
  configurable: true,
});
