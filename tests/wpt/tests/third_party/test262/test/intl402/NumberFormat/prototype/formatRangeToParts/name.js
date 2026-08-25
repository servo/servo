// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Intl.NumberFormat.prototype.formatRangeToParts.name value and descriptor.
includes: [propertyHelper.js]
features: [Intl.NumberFormat-v3]
---*/
verifyProperty(Intl.NumberFormat.prototype.formatRangeToParts, 'name', {
  value: 'formatRangeToParts',
  enumerable: false,
  writable: false,
  configurable: true,
});
