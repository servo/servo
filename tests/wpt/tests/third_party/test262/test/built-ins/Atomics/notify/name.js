// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Atomics.notify.name is "notify".
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics.notify, 'name', {
  value: 'notify',
  enumerable: false,
  writable: false,
  configurable: true,
});
