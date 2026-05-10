// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: Property descriptor of Intl.DurationFormat.prototype.formatToParts
includes: [propertyHelper.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(
  typeof Intl.DurationFormat.prototype.formatToParts,
  'function',
  '`typeof Intl.DurationFormat.prototype.formatToParts` is `function`'
);

verifyProperty(Intl.DurationFormat.prototype, 'formatToParts', {
  enumerable: false,
  writable: true,
  configurable: true,
});
