// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.resolvedOptions
description: Property descriptor of Intl.DurationFormat.prototype.resolvedOptions
includes: [propertyHelper.js]
features: [Intl.DurationFormat]
---*/

assert.sameValue(
  typeof Intl.DurationFormat.prototype.resolvedOptions,
  'function',
  '`typeof Intl.DurationFormat.prototype.resolvedOptions` is `function`'
);

verifyProperty(Intl.DurationFormat.prototype, 'resolvedOptions', {
  enumerable: false,
  writable: true,
  configurable: true,
});
