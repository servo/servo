// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.plainDateTimeISO
description: Temporal.Now.plainDateTimeISO.name is "plainDateTimeISO".
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  Temporal.Now.plainDateTimeISO.name,
  'plainDateTimeISO',
  'The value of Temporal.Now.plainDateTimeISO.name is expected to be "plainDateTimeISO"'
);

verifyProperty(Temporal.Now.plainDateTimeISO, 'name', {
  enumerable: false,
  writable: false,
  configurable: true
});
