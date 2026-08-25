// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.timezoneid
description: Temporal.Now.timeZoneId.name is "timeZoneId".
info: |
  ## 17 ECMAScript Standard Built-in Objects:
  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String.

  Unless otherwise specified, the name property of a built-in Function object,
  if it exists, has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  Temporal.Now.timeZoneId.name,
  'timeZoneId',
  'The value of Temporal.Now.timeZoneId.name is expected to be "timeZoneId"'
);

verifyProperty(Temporal.Now.timeZoneId, 'name', {
  enumerable: false,
  writable: false,
  configurable: true
});
