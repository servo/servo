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

const validFirstDayOfWeekOptions = [
  ["mon", 1],
  ["tue", 2],
  ["wed", 3],
  ["thu", 4],
  ["fri", 5],
  ["sat", 6],
  ["sun", 7],
  ["1", 1],
  ["2", 2],
  ["3", 3],
  ["4", 4],
  ["5", 5],
  ["6", 6],
  ["7", 7],
  ["0", 7],
  [1, 1],
  [2, 2],
  [3, 3],
  [4, 4],
  [5, 5],
  [6, 6],
  [7, 7],
  [0, 7],
];
for (const [firstDayOfWeek, expected] of validFirstDayOfWeekOptions) {
  assert.sameValue(
    new Intl.Locale('en', { firstDayOfWeek }).getWeekInfo().firstDay,
    expected,
    `new Intl.Locale("en", { firstDayOfWeek: ${firstDayOfWeek} }).getWeekInfo().firstDay returns "${expected}"`
  );
  assert.sameValue(
    new Intl.Locale('en-u-fw-WED', { firstDayOfWeek }).getWeekInfo().firstDay,
    expected,
    `new Intl.Locale("en-u-fw-WED", { firstDayOfWeek: ${firstDayOfWeek} }).firstDayOfWeek returns "${expected}"`
  );
}
