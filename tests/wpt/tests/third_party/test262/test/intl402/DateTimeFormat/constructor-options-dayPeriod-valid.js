// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
  Checks handling of the options argument to the DateTimeFormat constructor.
info: |
  [[DayPeriod]]    `"dayPeriod"`    `"narrow"`, `"short"`, `"long"`
  CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )

  ...
features: [Intl.DateTimeFormat-dayPeriod]
---*/

const validOptions = [
  [undefined, undefined],
  ["long", "long"],
  ["short", "short"],
  ["narrow", "narrow"],
  [{ toString() { return "narrow"; } }, "narrow"],
  [{ valueOf() { return "long"; }, toString: undefined }, "long"],
];
for (const [dayPeriod, expected] of validOptions) {
  const dtf = new Intl.DateTimeFormat("en", { dayPeriod });
  const options = dtf.resolvedOptions();
  assert.sameValue(options.dayPeriod, expected);
  const propdesc = Object.getOwnPropertyDescriptor(options, "dayPeriod");
  if (expected === undefined) {
    assert.sameValue(propdesc, undefined);
  } else {
    assert.sameValue(propdesc.value, expected);
  }
}
