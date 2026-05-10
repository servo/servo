// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Tests what information is present by default
locale: [en]
features: [Temporal]
---*/

const instant = new Temporal.Instant(1735213600_321_000_000n); // 2024-12-26T11:46:40.321Z
const result = instant.toLocaleString("en", { timeZone: "UTC" });

assert(result.includes("2024"), `Instant formatted with no options ${result} should include year`);
assert(result.includes("12") || result.includes("Dec"), `Instant formatted with no options ${result} should include month`);
assert(result.includes("26"), `Instant formatted with no options ${result} should include day`);
assert(result.includes("11"), `Instant formatted with no options ${result} should include hour`);
assert(result.includes("46"), `Instant formatted with no options ${result} should include minute`);
assert(result.includes("40"), `Instant formatted with no options ${result} should include second`);
assert(!result.includes("321"), `Instant formatted with no options ${result} should not include fractional second digits`);
assert(
  !result.includes("UTC") && !result.includes("Coordinated Universal Time"),
  `Instant formatted with no options ${result} should not include time zone name`
);
