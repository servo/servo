// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Unrecognized properties (incl. plurals of recognized units) are ignored
esid: sec-temporal.plaindatetime.prototype.with
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

assert.throws(
  TypeError,
  () => instance.with({}),
  "empty object not acceptable"
);

const units = ["year", "month", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];

units.forEach((unit) => {
  let plural = `${unit}s`;
  let options = {};
  options[plural] = 1;
  assert.throws(
    TypeError,
    () => instance.with(options),
    `plural unit ("${plural}" vs "${unit}") is not acceptable`
  );
});

assert.throws(
  TypeError,
  () => instance.with({nonsense: true}),
  "throw if no recognized properties present"
);

TemporalHelpers.assertPlainDateTime(
  instance.with({year: 1965, nonsense: true}),
  1965, 5, "M05", 2, 12, 34, 56, 987, 654, 321,
  "unrecognized properties ignored & does not throw if recognized properties present)"
);
