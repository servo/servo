// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: Tests what information is present by default
locale: [en]
features: [Temporal]
---*/

const plainDateTime = new Temporal.PlainDateTime(2024, 12, 26, 11, 46, 40, 321);
const result = plainDateTime.toLocaleString("en", { timeZone: "UTC" });

assert(result.includes("2024"), `PlainDateTime formatted with no options ${result} should include year`);
assert(result.includes("12") || result.includes("Dec"), `PlainDateTime formatted with no options ${result} should include month`);
assert(result.includes("26"), `PlainDateTime formatted with no options ${result} should include day`);
assert(result.includes("11"), `PlainDateTime formatted with no options ${result} should include hour`);
assert(result.includes("46"), `PlainDateTime formatted with no options ${result} should include minute`);
assert(result.includes("40"), `PlainDateTime formatted with no options ${result} should include second`);
assert(!result.includes("321"), `PlainDateTime formatted with no options ${result} should not include fractional second digits`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `PlainDateTime formatted with no options ${result} should not include time zone name`
);
