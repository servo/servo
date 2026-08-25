// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: Property descriptor of Intl.PluralRules.prototype.selectRange
includes: [propertyHelper.js]
features: [Intl.NumberFormat-v3]
---*/

assert.sameValue(
  typeof Intl.PluralRules.prototype.selectRange,
  'function',
  '`typeof Intl.PluralRules.prototype.selectRange` is `function`'
);

verifyProperty(Intl.PluralRules.prototype, 'selectRange', {
  enumerable: false,
  writable: true,
  configurable: true,
});
