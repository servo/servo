// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Month codes that are invalid for Hebrew calendar
features: [Temporal, Intl.Era-monthcode]
---*/

// The Hebrew calendar is a 12-month lunisolar calendar with leap month M05L
// (Adar I) but does not have a thirteenth month (M13)

const calendar = "hebrew";

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

// Invalid leap months: e.g. M02L
for (var i = 1; i <= 12; i++) {
  if (i === 5)
    continue;
  const monthCode = `M${ i.toString().padStart(2, "0") }L`;
  assert.throws(RangeError, function () {
    Temporal.PlainMonthDay.from({ monthCode, day: 1, calendar });
  });
}
