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

const validFirstDayOfWeekOptions = [
  ["mon", "mon"],
  ["tue", "tue"],
  ["wed", "wed"],
  ["thu", "thu"],
  ["fri", "fri"],
  ["sat", "sat"],
  ["sun", "sun"],
  ["1", "mon"],
  ["2", "tue"],
  ["3", "wed"],
  ["4", "thu"],
  ["5", "fri"],
  ["6", "sat"],
  ["7", "sun"],
  ["0", "sun"],
  [1, "mon"],
  [2, "tue"],
  [3, "wed"],
  [4, "thu"],
  [5, "fri"],
  [6, "sat"],
  [7, "sun"],
  [0, "sun"],
  [-0, "sun"],
  [100, "100"],
  [1000, "1000"],
  [1n, "mon"],
  [2n, "tue"],
  [3n, "wed"],
  [4n, "thu"],
  [5n, "fri"],
  [6n, "sat"],
  [7n, "sun"],
  [0n, "sun"],
  [100n, "100"],
  [1000n, "1000"],
  [NaN, "nan"],
  [Infinity, "infinity"],
  [true, ""],
  [false, "false"],
  [null, "null"],
  ["yes", "yes"],
  ["lunedi", "lunedi"],
  ["montag", "montag"],
];
for (const [firstDayOfWeek, expected] of validFirstDayOfWeekOptions) {
  assert.sameValue(
    new Intl.Locale("en", { firstDayOfWeek }).firstDayOfWeek,
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${firstDayOfWeek} }).firstDayOfWeek returns "${expected}"`
  );
  assert.sameValue(
    new Intl.Locale("en-u-fw-WED", { firstDayOfWeek }).firstDayOfWeek,
    expected,
    `new Intl.Locale("en-u-fw-WED", { firstDayOfWeek: ${firstDayOfWeek} }).firstDayOfWeek returns "${expected}"`
  );

  let upperCase = String(firstDayOfWeek).toUpperCase();
  assert.sameValue(
    new Intl.Locale("en", { firstDayOfWeek: upperCase }).firstDayOfWeek,
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${upperCase} }).firstDayOfWeek returns "${expected}"`
  );
}
