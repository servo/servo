// Copyright 2023 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks valid cases for the locale argument to the Locale constructor.
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

  ["en-u-fw", ""],
  ["en-u-fw-true", ""],
  ["en-u-fw-false", "false"],
  ["en-u-fw-yes", "yes"],
  ["en-u-fw-lunedi", "lunedi"],
  ["en-u-fw-montag", "montag"],

  ["en-u-fw-000", "000"],
  ["en-u-fw-001", "001"],
  ["en-u-fw-11111111", "11111111"],
];
for (const [id, expected] of validIds) {
  assert.sameValue(
    new Intl.Locale(id).firstDayOfWeek,
    expected,
    `new Intl.Locale("${id}").firstDayOfWeek returns "${expected}"`
  );

  let upperCase = id.toUpperCase();
  assert.sameValue(
    new Intl.Locale(upperCase).firstDayOfWeek,
    expected,
    `new Intl.Locale("${upperCase}").firstDayOfWeek returns "${expected}"`
  );
}
