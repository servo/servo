// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Tests what information is present by default
locale: [en-u-ca-iso8601]
features: [Temporal]
---*/

const plainYearMonth = new Temporal.PlainYearMonth(2024, 12, "iso8601", 26);  // nonstandard reference day
const result = plainYearMonth.toLocaleString("en-u-ca-iso8601", { timeZone: "UTC" });

assert(result.includes("2024"), `PlainYearMonth formatted with no options ${result} should include year`);
assert(result.includes("12") || result.includes("Dec"), `PlainYearMonth formatted with no options ${result} should include month`);
assert(!result.includes("26"), `PlainYearMonth formatted with no options ${result} should not include reference day`);
assert(!result.includes("00"), `PlainYearMonth formatted with no options ${result} should not include hour, minute, second`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `PlainYearMonth formatted with no options ${result} should not include time zone name`
);
