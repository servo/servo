// Copyright 2016 Mozilla Corporation. All rights reserved.
// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Property type and descriptor.
includes: [propertyHelper.js]
features: [Intl.DateTimeFormat-formatRange]
---*/

assert.sameValue(
  typeof Intl.DateTimeFormat.prototype.formatRange,
  'function',
  '`typeof Intl.DateTimeFormat.prototype.formatRange` is `function`'
);

verifyProperty(Intl.DateTimeFormat.prototype, 'formatRange', {
  enumerable: false,
  writable: true,
  configurable: true,
});
