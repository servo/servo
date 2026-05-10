// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Duration-like argument that is out of range
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1970, 1);

const cases = [
  // 2^32 = 4294967296
  ["P4294967296Y", "string with years > max"],
  [{ years: 4294967296 }, "property bag with years > max"],
  ["-P4294967296Y", "string with years < min"],
  [{ years: -4294967296 }, "property bag with years < min"],
  ["P4294967296M", "string with months > max"],
  [{ months: 4294967296 }, "property bag with months > max"],
  ["-P4294967296M", "string with months < min"],
  [{ months: -4294967296 }, "property bag with months < min"],
  ["P4294967296W", "string with weeks > max"],
  [{ weeks: 4294967296 }, "property bag with weeks > max"],
  ["-P4294967296W", "string with weeks < min"],
  [{ weeks: -4294967296 }, "property bag with weeks < min"],
];

for (const [arg, descr] of cases) {
  assert.throws(RangeError, () => instance.add(arg), `${descr} is out of range`);
}
