// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: >
    Verifies there's no branding check for Intl.DurationFormat.supportedLocalesOf().
info: |
    Intl.DurationFormat.supportedLocalesOf ( locales [, options ])
features: [Intl.DurationFormat]
---*/

const supportedLocalesOf = Intl.DurationFormat.supportedLocalesOf;

assert.sameValue(typeof supportedLocalesOf, "function");

const thisValues = [
  undefined,
  null,
  true,
  "",
  Symbol(),
  1,
  {},
  Intl.DurationFormat,
  Intl.DurationFormat.prototype,
];

for (const thisValue of thisValues) {
  const result = supportedLocalesOf.call(thisValue);
  assert.sameValue(Array.isArray(result), true);
}
