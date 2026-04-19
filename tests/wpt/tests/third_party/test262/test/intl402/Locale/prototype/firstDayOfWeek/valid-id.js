// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the options argument to the Locale constructor.
info: |
    Intl.Locale.prototype.firstDayOfWeek
    3. Return loc.[[FirstDayOfWeek]].

features: [Intl.Locale,Intl.Locale-info]
---*/

const validIds = [
  ["en-u-fw-mon", "mon"],
  ["en-u-fw-tue", "tue"],
  ["en-u-fw-wed", "wed"],
  ["en-u-fw-thu", "thu"],
  ["en-u-fw-fri", "fri"],
  ["en-u-fw-sat", "sat"],
  ["en-u-fw-sun", "sun"],
];
for (const [id, expected] of validIds) {
  assert.sameValue(
    new Intl.Locale(id).firstDayOfWeek,
    expected,
    `new Intl.Locale(${id}).firstDayOfWeek returns "${expected}"`
  );
}
