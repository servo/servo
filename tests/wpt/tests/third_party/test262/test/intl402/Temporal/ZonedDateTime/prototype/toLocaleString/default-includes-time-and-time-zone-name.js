// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: Tests what information is present by default
locale: [en]
features: [Temporal]
---*/

const zonedDateTime = new Temporal.ZonedDateTime(1735213600_321_000_000n, "UTC"); // 2024-12-26T11:46:40.321Z
const result = zonedDateTime.toLocaleString("en");

assert(result.includes("2024"), `ZonedDateTime formatted with no options ${result} should include year`);
assert(result.includes("12") || result.includes("Dec"), `ZonedDateTime formatted with no options ${result} should include month`);
assert(result.includes("26"), `ZonedDateTime formatted with no options ${result} should include day`);
assert(result.includes("11"), `ZonedDateTime formatted with no options ${result} should include hour`);
assert(result.includes("46"), `ZonedDateTime formatted with no options ${result} should include minute`);
assert(result.includes("40"), `ZonedDateTime formatted with no options ${result} should include second`);
assert(!result.includes("321"), `ZonedDateTime formatted with no options ${result} should not include fractional second digits`);
assert(
  result.includes("UTC") || result.includes("Coordinated Universal Time"),
  `ZonedDateTime formatted with no options ${result} should include time zone name`
);
