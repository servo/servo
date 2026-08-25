// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Verify that the result of ToTemporalMonthDay preserves year information for Non-ISO calendars.
info: |
    sec-temporal.plainmonthday.from step 3:
      3. Return ? ToTemporalMonthDay(_item_, _options_).
    sec-temporal-totemporalmonthday step 11.:
      11. Set result to ? CreateTemporalMonthDay(_result_.[[Month]], _result_.[[Day]], _calendar_, _result_.[[Year]]).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const pmd = Temporal.PlainMonthDay.from("2023-01-01[u-ca=hebrew]")
TemporalHelpers.assertPlainMonthDay(pmd, "M04", 8); // 2023-01-01 corresponds to 8 Tevet in Hebrew Calendar.
