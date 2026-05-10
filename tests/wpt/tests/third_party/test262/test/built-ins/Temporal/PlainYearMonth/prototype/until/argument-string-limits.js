// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: ISO strings at the edges of the representable range
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1970, 1);

// Note, these limits are different than other PlainYearMonth conversions
// because the difference is taken between the first days of the two months, so
// the first day of the month of the argument must be within the representable
// range

const validStrings = [
  "-271821-05",
  "-271821-05-01",
  "-271821-05-01T00:00",
  "+275760-09",
  "+275760-09-30",
  "+275760-09-30T23:59:59.999999999",
];

for (const arg of validStrings) {
  instance.until(arg);
}

const invalidStrings = [
  "-271821-04",
  "-271821-04-30",
  "-271821-04-30T23:59:59.999999999",
  "+275760-10",
  "+275760-10-01",
  "+275760-10-01T00:00",
];

for (const arg of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `"${arg}" is outside the representable range of PlainYearMonth`
  );
}
