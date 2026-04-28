// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.supportedLocalesOf
description: >
    Verifies there's no branding check for Intl.ListFormat.supportedLocalesOf().
info: |
    Intl.ListFormat.supportedLocalesOf ( locales [, options ])
features: [Intl.ListFormat]
---*/

const fn = Intl.ListFormat.supportedLocalesOf;
const thisValues = [
  undefined,
  null,
  true,
  "",
  Symbol(),
  1,
  {},
  Intl.ListFormat,
  Intl.ListFormat.prototype,
];

for (const thisValue of thisValues) {
  const result = fn.call(thisValue);
  assert.sameValue(Array.isArray(result), true);
}
