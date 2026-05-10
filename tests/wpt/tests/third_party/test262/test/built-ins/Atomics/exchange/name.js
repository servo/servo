// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.exchange
description: >
  Atomics.exchange.name is "exchange".
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics.exchange, 'name', {
  value: 'exchange',
  enumerable: false,
  writable: false,
  configurable: true,
});
