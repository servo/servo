// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.instant
description: Temporal.Now.instant.name is "instant".
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  Temporal.Now.instant.name,
  'instant',
  'The value of Temporal.Now.instant.name is expected to be "instant"'
);

verifyProperty(Temporal.Now.instant, 'name', {
  enumerable: false,
  writable: false,
  configurable: true
});
