// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.compareexchange
description: >
  Atomics.compareExchange.name is "compareExchange".
includes: [propertyHelper.js]
features: [Atomics]
---*/

verifyProperty(Atomics.compareExchange, 'name', {
  value: 'compareExchange',
  enumerable: false,
  writable: false,
  configurable: true,
});
