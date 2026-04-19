// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: Tests what information is present by default
locale: [en-u-ca-iso8601]
features: [Temporal]
---*/

const plainMonthDay = new Temporal.PlainMonthDay(12, 26);
const result = plainMonthDay.toLocaleString("en-u-ca-iso8601", { timeZone: "UTC" });

assert(!result.includes("1972"), `PlainMonthDay formatted with no options ${result} should not include reference year`);
assert(result.includes("12") || result.includes("Dec"), `PlainMonthDay formatted with no options ${result} should include month`);
assert(result.includes("26"), `PlainMonthDay formatted with no options ${result} should include day`);
assert(!result.includes("00"), `PlainMonthDay formatted with no options ${result} should not include hour, minute, second`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `PlainMonthDay formatted with no options ${result} should not include time zone name`
);
