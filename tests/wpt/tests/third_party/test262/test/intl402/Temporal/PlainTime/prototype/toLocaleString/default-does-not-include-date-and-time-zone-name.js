// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Tests what information is present by default
locale: [en]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(11, 46, 40, 321);
const result = plainTime.toLocaleString("en", { timeZone: "UTC" });

assert(!result.includes("1970"), `PlainTime formatted with no options ${result} should not include year`);
assert(!result.includes("01") && !result.includes("Jan"), `PlainTime formatted with no options ${result} should not include month or day`);
assert(result.includes("11"), `PlainTime formatted with no options ${result} should include hour`);
assert(result.includes("46"), `PlainTime formatted with no options ${result} should include minute`);
assert(result.includes("40"), `PlainTime formatted with no options ${result} should include second`);
assert(!result.includes("321"), `PlainTime formatted with no options ${result} should not include fractional second digits`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `PlainTime formatted with no options ${result} should not include time zone name`
);
