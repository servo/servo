// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Property descriptor of Intl.DurationFormat.prototype.format
includes: [propertyHelper.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(
  typeof Intl.DurationFormat.prototype.format,
  'function',
  '`typeof Intl.DurationFormat.prototype.format` is `function`'
);

verifyProperty(Intl.DurationFormat.prototype, 'format', {
  enumerable: false,
  writable: true,
  configurable: true,
});
