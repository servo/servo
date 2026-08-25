// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: Time separator in string argument can vary
features: [Temporal]
---*/

const tests = [
  ["2019-12-15T15:23", "uppercase T"],
  ["2019-12-15t15:23", "lowercase T"],
  ["2019-12-15 15:23", "space between date and time"],
];

const instance = new Temporal.PlainYearMonth(2019, 12);

tests.forEach(([arg, description]) => {
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `variant time separators (${description})`
  );
});
