// Copyright 2023 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getWeekInfo
description: >
    Checks that the return value of Intl.Locale.prototype.getWeekInfo is an Object
    with the correct keys and properties.
info: |
  get Intl.Locale.prototype.getWeekInfo
  ...
  6. Perform ! CreateDataPropertyOrThrow(info, "firstDay", wi.[[FirstDay]]).
features: [Reflect,Intl.Locale,Intl.Locale-info]
---*/

const validFirstDayOfWeekIds = [
  ["en-u-fw-mon", 1],
  ["en-u-fw-tue", 2],
  ["en-u-fw-wed", 3],
  ["en-u-fw-thu", 4],
  ["en-u-fw-fri", 5],
  ["en-u-fw-sat", 6],
  ["en-u-fw-sun", 7],
];
for (const [id, expected] of validFirstDayOfWeekIds) {
  assert.sameValue(
    new Intl.Locale(id).getWeekInfo().firstDay,
    expected,
    `new Intl.Locale(${id}).getWeekInfo().firstDay returns "${expected}"`
  );
}
