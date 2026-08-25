// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: M13 month code is invalid for Japanese calendar (12-month calendar)
features: [Temporal, Intl.Era-monthcode]
---*/

// The Japanese calendar is a 12-month calendar and should not accept M13

const calendar = "japanese";

assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M13", day: 1 });
}, `M13 should not be a valid month code for ${calendar} calendar`);

// M13 should throw even with overflow: "constrain"
assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M13", day: 1 }, { overflow: "constrain" });
}, `M13 should not be valid for ${calendar} calendar even with constrain overflow`);

// M13 should throw with overflow: "reject"
assert.throws(RangeError, () => {
  Temporal.PlainMonthDay.from({ calendar, monthCode: "M13", day: 1 }, { overflow: "reject" });
}, `M13 should not be valid for ${calendar} calendar with reject overflow`);
