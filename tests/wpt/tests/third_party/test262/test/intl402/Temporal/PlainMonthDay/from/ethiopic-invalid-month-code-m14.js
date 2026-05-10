// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: M14 month code is invalid for Ethiopic calendar (13-month calendar)
features: [Temporal, Intl.Era-monthcode]
---*/

// The Ethiopic calendar has 13 months (M01-M13) and should not accept M14

const calendar = "ethiopic";

assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M14", day: 1 });
}, `M14 should not be a valid month code for ${calendar} calendar`);

// M14 should throw even with overflow: "constrain"
assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M14", day: 1 }, { overflow: "constrain" });
}, `M14 should not be valid for ${calendar} calendar even with constrain overflow`);

// M14 should throw with overflow: "reject"
assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M14", day: 1 }, { overflow: "reject" });
}, `M14 should not be valid for ${calendar} calendar with reject overflow`);
