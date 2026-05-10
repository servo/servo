// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Validation of monthCode
features: [Temporal]
---*/

["m1", "M1", "m01"].forEach((monthCode) => {
  assert.throws(RangeError, () => Temporal.PlainDateTime.from({ year: 2021, monthCode, day: 17 }),
    `monthCode '${monthCode}' is not well-formed`);
});

assert.throws(RangeError, () => Temporal.PlainDateTime.from({ year: 2021, month: 12, monthCode: "M11", day: 17 }),
  "monthCode and month conflict");

["M00", "M19", "M99", "M13", "M00L", "M05L", "M13L"].forEach((monthCode) => {
  assert.throws(RangeError, () => Temporal.PlainDateTime.from({ year: 2021, monthCode, day: 17 }),
    `monthCode '${monthCode}' is not valid for ISO 8601 calendar`);
});

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from({ day: 1, monthCode: "L99M", year: Symbol() }),
  "Month code syntax is validated before year type is validated"
);

assert.throws(
  TypeError,
  () => Temporal.PlainDateTime.from({ day: 1, monthCode: "M99L", year: Symbol() }),
  "Month code suitability is validated after year type is validated"
);
