// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: Intl.PluralRules.prototype.selectRange.name is "selectRange"
includes: [propertyHelper.js]
features: [Intl.NumberFormat-v3]
---*/

verifyProperty(Intl.PluralRules.prototype.selectRange, 'name', {
  value: 'selectRange',
  enumerable: false,
  writable: false,
  configurable: true,
});

